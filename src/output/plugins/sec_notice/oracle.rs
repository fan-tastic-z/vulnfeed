use async_trait::async_trait;
use error_stack::ResultExt;
use mea::mpsc::UnboundedSender;
use scraper::{Html, Selector};

use crate::{
    AppResult,
    domain::models::security_notice::{CreateSecurityNotice, RiskLevel},
    errors::Error,
    output::plugins::sec_notice::{SecNoticePlugin, register_sec_notice},
    utils::http_client::HttpClient,
};

const ORACLE_NOTICE_URL: &str = "https://www.oracle.com/cn/security-alerts/";

#[derive(Debug, Clone)]
pub struct OracleNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for OracleNoticePlugin {
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
        let notices = self.parse_security_notices().await?;
        for notice in notices {
            let create_security_notice = CreateSecurityNotice {
                key: notice.unique_id.clone(),
                title: notice.title,
                product_name: "Oracle".to_string(),
                risk_level: RiskLevel::Critical.to_string(),
                source: self.link.clone(),
                source_name: self.get_name(),
                is_zero_day: false,
                detail_link: notice.detail_link,
                publish_time: notice.publish_time,
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

impl OracleNoticePlugin {
    pub fn try_new(sender: UnboundedSender<CreateSecurityNotice>) -> AppResult<OracleNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let oracle = OracleNoticePlugin {
            name: "OraclePlugin".to_string(),
            display_name: "Oracle安全公告".to_string(),
            link: ORACLE_NOTICE_URL.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(oracle.name.clone(), Box::new(oracle.clone()));
        Ok(oracle)
    }

    async fn get_document(&self, url: &str) -> AppResult<Html> {
        let content = self.http_client.get_html_content(url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    /// 解析安全公告信息
    ///
    /// 该方法会从Oracle安全公告页面提取"Critical Patch Update"表格中的前三行数据。
    ///
    /// # 返回值
    /// 返回解析到的安全公告列表或错误信息
    pub async fn parse_security_notices(&self) -> AppResult<Vec<OracleSecurityNotice>> {
        let document = self.get_document(ORACLE_NOTICE_URL).await?;

        // 选择包含"Critical Patch Updates"的表格
        let table_selector = Selector::parse("table.otable-tech-basic").map_err(|e| {
            Error::Message(format!("Failed to parse CSS selector for table: {}", e))
        })?;

        let mut notices = Vec::new();

        // 查找第一个表格（Critical Patch Updates表格）
        if let Some(table) = document.select(&table_selector).next() {
            let row_selector = Selector::parse("tbody tr").map_err(|e| {
                Error::Message(format!("Failed to parse CSS selector for rows: {}", e))
            })?;

            let rows: Vec<_> = table.select(&row_selector).collect();

            // 获取前三行数据（跳过可能的空行）
            let mut count = 0;
            for row in rows {
                // 跳过空行
                if row
                    .select(&Selector::parse("td").map_err(|e| {
                        Error::Message(format!("Failed to parse CSS selector for td: {}", e))
                    })?)
                    .count()
                    == 0
                {
                    continue;
                }

                if count >= 3 {
                    break;
                }

                let td_selector = Selector::parse("td").map_err(|e| {
                    Error::Message(format!("Failed to parse CSS selector for td: {}", e))
                })?;

                let td_elements: Vec<_> = row.select(&td_selector).collect();

                // 确保行中有足够的列
                if td_elements.len() >= 2 {
                    // 提取链接和标题
                    let title_element = td_elements[0]
                        .select(&Selector::parse("a").map_err(|e| {
                            Error::Message(format!("Failed to parse CSS selector for a: {}", e))
                        })?)
                        .next();

                    if let Some(title_el) = title_element {
                        let original_title = title_el.text().collect::<String>().trim().to_string();
                        let href = title_el.value().attr("href").unwrap_or("").to_string();
                        let detail_link = format!("https://www.oracle.com{}", href);

                        // 提取发布日期和版本信息 ("Rev 4, 28 July 2025")
                        let version_date_text =
                            td_elements[1].text().collect::<String>().trim().to_string();

                        // 解析版本号 ("Rev 4") 和发布日期 ("28 July 2025")
                        let (version, publish_time) = if let Some(pos) = version_date_text.find(',')
                        {
                            let version_part = version_date_text[..pos].trim().to_string();
                            let date_part = version_date_text[pos + 1..].trim().to_string();
                            (version_part, date_part)
                        } else {
                            continue;
                        };

                        // 构造新标题 ("Critical Patch Update + Rev 4")
                        let title = format!("{} {}", original_title, version);

                        // 从href中提取HTML文件名并构造唯一标识符
                        let unique_id = {
                            let path_segments: Vec<&str> = detail_link.split('/').collect();
                            if let Some(filename) = path_segments.last() {
                                let html_name = filename.trim_end_matches(".html");
                                format!("{}-{}", html_name, version)
                            } else {
                                continue;
                            }
                        };

                        notices.push(OracleSecurityNotice {
                            title,
                            detail_link,
                            publish_time,
                            unique_id,
                        });

                        count += 1;
                    }
                }
            }
        }

        Ok(notices)
    }
}

#[derive(Debug, Clone)]
pub struct OracleSecurityNotice {
    pub title: String,
    pub detail_link: String,
    pub publish_time: String,
    pub unique_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mea::mpsc::unbounded;

    #[tokio::test]
    async fn test_parse_security_notices() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let plugin = OracleNoticePlugin::try_new(sender).unwrap();
        let notices = plugin.parse_security_notices().await.unwrap();
        // 检查是否获取到了至少3个公告
        assert!(notices.len() >= 3);
    }
}
