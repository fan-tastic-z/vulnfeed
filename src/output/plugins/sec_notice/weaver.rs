use crate::domain::models::security_notice::RiskLevel;
use crate::errors::Error;
use crate::output::plugins::sec_notice::{SecNoticePlugin, register_sec_notice};
use crate::utils::http_client::HttpClient;
use crate::{AppResult, domain::models::security_notice::CreateSecurityNotice};
use async_trait::async_trait;
use error_stack::ResultExt;
use mea::mpsc::UnboundedSender;
use scraper::{Html, Selector};
use std::io::Cursor;
use zip::ZipArchive;

const PATCH_LINKS: &str = "https://www.weaver.com.cn/cs/securityDownload.html?src=cn";

#[derive(Debug, Clone)]
pub struct WeaverNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for WeaverNoticePlugin {
    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn get_display_name(&self) -> String {
        self.display_name.to_string()
    }

    fn get_link(&self) -> String {
        self.link.to_string()
    }

    async fn update(&self, _page_limit: i32) -> AppResult<()> {
        let patches = self.parse_security_notices().await?;
        if patches.len() < 3 {
            return Err(Error::Message("Not enough patches found".to_string()).into());
        }
        for patch in patches {
            // "EC9.0全量补丁", "EC8.0全量补丁", "EC10.0安全补丁"
            let product_name = match patch.title.as_str() {
                "EC9.0全量补丁" => "EC9.0",
                "EC8.0全量补丁" => "EC8.0",
                "EC10.0安全补丁" => "EC10.0",
                _ => return Err(Error::Message("Invalid product name".to_string()).into()),
            };
            let create_security_notice = CreateSecurityNotice {
                key: patch.md5,
                title: patch.title,
                product_name: product_name.to_string(),
                risk_level: RiskLevel::Critical.to_string(),
                source: self.link.clone(),
                source_name: self.get_name(),
                is_zero_day: true, // 这里默认都是true
                publish_time: patch.update_time,
                detail_link: PATCH_LINKS.to_string(),
                pushed: false,
            };
            self.sender
                .send(create_security_notice)
                .change_context_lazy(|| {
                    Error::Message("Failed to send security notice to queue".to_string())
                })?;
        }
        Ok(())
    }
}

impl WeaverNoticePlugin {
    pub fn try_new(sender: UnboundedSender<CreateSecurityNotice>) -> AppResult<WeaverNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let weaver = WeaverNoticePlugin {
            name: "WeaverPlugin".to_string(),
            display_name: "泛微ECOLOGY安全补丁包".to_string(),
            link: PATCH_LINKS.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(weaver.name.clone(), Box::new(weaver.clone()));
        Ok(weaver)
    }

    async fn get_document(&self, url: &str) -> AppResult<Html> {
        let content = self.http_client.get_html_content(url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    pub async fn get_patch_md5(&self, link: &str) -> AppResult<String> {
        let content = self.download_and_extract_zip(link).await?;
        if let Some(first_line) = content.lines().next() {
            // 目前文件中的冒号是中文冒号，防止后面更改，兼容英文冒号做处理
            let second_value = if first_line.contains('：') {
                first_line.split('：').nth(1).map(|s| s.trim())
            } else {
                first_line.split(':').nth(1).map(|s| s.trim())
            };
            if let Some(v) = second_value {
                return Ok(v.to_lowercase());
            } else {
                return Err(Error::Message("Invalid patch content".to_string()).into());
            }
        }
        Err(Error::Message("Invalid patch content".to_string()).into())
    }

    pub async fn download_and_extract_zip(&self, url: &str) -> AppResult<String> {
        let response =
            self.http_client.get(url).await.change_context_lazy(|| {
                Error::Message("Failed to download ZIP file".to_string())
            })?;

        let b = response
            .bytes()
            .await
            .change_context_lazy(|| Error::Message("Failed to read response body".to_string()))?;

        let cursor = Cursor::new(b);
        let mut archive = ZipArchive::new(cursor)
            .change_context_lazy(|| Error::Message("Failed to parse ZIP archive".to_string()))?;

        let first_file_name = {
            let names: Vec<_> = archive.file_names().map(|s| s.to_owned()).collect();
            names.into_iter().next()
        };
        if let Some(file_name) = first_file_name {
            let mut file = archive.by_name(&file_name).change_context_lazy(|| {
                Error::Message("Failed to read file from ZIP archive".to_string())
            })?;
            let mut contents = Vec::new();
            std::io::copy(&mut file, &mut contents).change_context_lazy(|| {
                Error::Message("Failed to copy file contents".to_string())
            })?;
            Ok(String::from_utf8_lossy(&contents).to_string())
        } else {
            Err(Error::Message("ZIP file is empty".to_string()).into())
        }
    }

    pub async fn parse_security_notices(&self) -> AppResult<Vec<WeaverSecurityNotice>> {
        let document = self.get_document(&self.link).await?;
        let selector = Selector::parse(".module-table tbody tr")
            .map_err(|e| Error::Message(format!("Failed to parse CSS selector: {}", e)))?;
        let tr_elements: Vec<_> = document.select(&selector).collect();

        let mut notices = Vec::new();

        // 目前只需要关注这三个补丁的数据
        let target_titles = ["EC9.0全量补丁", "EC8.0全量补丁", "EC10.0安全补丁"];

        // 提取所有需要的数据到 owned 类型中，避免在异步调用中持有 Element 引用
        let mut patch_data = Vec::new();

        for tr in tr_elements {
            let td_elements: Vec<_> = tr
                .select(
                    &Selector::parse("td").map_err(|e| {
                        Error::Message(format!("Failed to parse CSS selector: {}", e))
                    })?,
                )
                .collect();

            for td in td_elements {
                if let Some(title_element) = td
                    .select(
                        &Selector::parse("div.module-title.for-folder").map_err(|e| {
                            Error::Message(format!("Failed to parse CSS selector: {}", e))
                        })?,
                    )
                    .next()
                {
                    let title = title_element.text().collect::<String>().trim().to_string();

                    if target_titles.contains(&title.as_str()) {
                        let version = if let Some(version_element) = td
                            .select(&Selector::parse("div.module-note").map_err(|e| {
                                Error::Message(format!("Failed to parse CSS selector: {}", e))
                            })?)
                            .find(|el| el.text().any(|t| t.contains("官方版本")))
                        {
                            version_element
                                .text()
                                .collect::<String>()
                                .split('：')
                                .nth(1)
                                .unwrap_or("")
                                .split_whitespace()
                                .next()
                                .unwrap_or("")
                                .to_string()
                        } else {
                            String::new()
                        };

                        let update_time = if let Some(time_element) = td
                            .select(&Selector::parse("div.module-note span").map_err(|e| {
                                Error::Message(format!("Failed to parse CSS selector: {}", e))
                            })?)
                            .next()
                        {
                            time_element.text().collect::<String>().trim().to_string()
                        } else {
                            String::new()
                        };

                        let checksum_link = if let Some(link_element) = td
                            .select(&Selector::parse("div.module-note a").map_err(|e| {
                                Error::Message(format!("Failed to parse CSS selector: {}", e))
                            })?)
                            .next()
                        {
                            link_element.value().attr("href").unwrap_or("").to_string()
                        } else {
                            String::new()
                        };

                        patch_data.push((title, version, update_time, checksum_link));
                    }
                }
            }
        }

        // 处理每个补丁的数据
        for (title, version, update_time, checksum_link) in patch_data {
            // EC9.0全量补丁 and EC8.0全量补丁, get MD5 value
            let md5 = if ("EC9.0全量补丁" == title || "EC8.0全量补丁" == title)
                && !checksum_link.is_empty()
            {
                let full_url = format!("https://www.weaver.com.cn/cs/{}", checksum_link);
                (self.get_patch_md5(&full_url).await).unwrap_or_default()
            } else if "EC10.0安全补丁" == title {
                // 重新解析文档以获取 MD5 值
                let document = self.get_document(&self.link).await?;
                let selector = Selector::parse(".module-table tbody tr")
                    .map_err(|e| Error::Message(format!("Failed to parse CSS selector: {}", e)))?;
                let tr_elements: Vec<_> = document.select(&selector).collect();

                let mut md5_value = String::new();
                'outer: for tr in tr_elements {
                    let td_elements: Vec<_> = tr
                        .select(&Selector::parse("td").map_err(|e| {
                            Error::Message(format!("Failed to parse CSS selector: {}", e))
                        })?)
                        .collect();

                    for td in td_elements {
                        if let Some(title_element) = td
                            .select(&Selector::parse("div.module-title.for-folder").map_err(
                                |e| Error::Message(format!("Failed to parse CSS selector: {}", e)),
                            )?)
                            .next()
                        {
                            let current_title =
                                title_element.text().collect::<String>().trim().to_string();
                            if current_title == title
                                && let Some(md5_element) = td
                                    .select(&Selector::parse("div.module-note").map_err(|e| {
                                        Error::Message(format!(
                                            "Failed to parse CSS selector: {}",
                                            e
                                        ))
                                    })?)
                                    .find(|el| el.text().any(|t| t.contains("MD5")))
                            {
                                {
                                    md5_value = md5_element
                                        .text()
                                        .collect::<String>()
                                        .split('：')
                                        .nth(1)
                                        .unwrap_or("")
                                        .split_whitespace()
                                        .next()
                                        .unwrap_or("")
                                        .to_string();
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
                md5_value
            } else {
                String::new()
            }
            .to_lowercase();

            notices.push(WeaverSecurityNotice {
                title,
                version,
                update_time,
                checksum_link,
                md5,
            });
        }

        Ok(notices)
    }
}

#[derive(Debug, Clone)]
pub struct WeaverSecurityNotice {
    pub title: String,
    pub version: String,
    pub update_time: String,
    pub checksum_link: String,
    pub md5: String,
}
