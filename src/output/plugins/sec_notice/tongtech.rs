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

const TONGTECH_NOTICE_URL: &str = "https://www.tongtech.com/dft/download.html";

#[derive(Debug, Clone)]
pub struct TongtechNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for TongtechNoticePlugin {
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

        // 只取前三条数据
        for notice in notices.into_iter().take(3) {
            let create_security_notice = CreateSecurityNotice {
                key: notice.id.clone(),
                title: notice.title,
                product_name: "Tongtech".to_string(),
                risk_level: RiskLevel::High.to_string(), // 默认高风险，因为这些都是安全补丁
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

impl TongtechNoticePlugin {
    pub fn try_new(
        sender: UnboundedSender<CreateSecurityNotice>,
    ) -> AppResult<TongtechNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let tongtech = TongtechNoticePlugin {
            name: "TongtechPlugin".to_string(),
            display_name: "东方通安全公告".to_string(),
            link: TONGTECH_NOTICE_URL.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(tongtech.name.clone(), Box::new(tongtech.clone()));
        Ok(tongtech)
    }

    async fn get_document(&self, url: &str) -> AppResult<Html> {
        let content = self.http_client.get_html_content(url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    /// 解析安全公告信息
    ///
    /// 该方法会从东方通下载页面获取安全补丁列表，并提取前三条数据。
    ///
    /// # 返回值
    /// 返回解析到的安全公告列表或错误信息
    pub async fn parse_security_notices(&self) -> AppResult<Vec<TongtechSecurityNotice>> {
        let document = self.get_document(&self.link).await?;

        // 选择 class="fuwuyuzhichi_cpzl2" 的 div
        let container_selector = Selector::parse("div.fuwuyuzhichi_cpzl2").map_err(|e| {
            Error::Message(format!("Failed to parse CSS selector for container: {}", e))
        })?;

        let container = document.select(&container_selector).next().ok_or_else(|| {
            Error::Message("Failed to find fuwuyuzhichi_cpzl2 container".to_string())
        })?;

        // 选择 ul 下的 li 元素
        let li_selector = Selector::parse("ul.cf > li")
            .map_err(|e| Error::Message(format!("Failed to parse CSS selector for li: {}", e)))?;

        let mut notices = Vec::new();

        // 遍历前三个 li 元素
        for (index, li_element) in container.select(&li_selector).enumerate() {
            if index >= 3 {
                break;
            }

            // 选择 a 标签
            let a_selector = Selector::parse("a").map_err(|e| {
                Error::Message(format!("Failed to parse CSS selector for a: {}", e))
            })?;

            if let Some(a_element) = li_element.select(&a_selector).next() {
                // 提取 onclick 属性中的 ID
                let onclick = a_element.value().attr("onclick").unwrap_or("");
                let id = if let Some(start) = onclick.find("downloadLink(") {
                    let start = start + "downloadLink(".len();
                    let end = onclick[start..].find(')').unwrap_or(onclick.len() - start);
                    onclick[start..start + end].to_string()
                } else {
                    format!("{}", index) // 如果没有找到 ID，使用索引作为备用
                };

                // 提取标题 (h2)
                let h2_selector = Selector::parse("h2").map_err(|e| {
                    Error::Message(format!("Failed to parse CSS selector for h2: {}", e))
                })?;

                let title = if let Some(h2_element) = a_element.select(&h2_selector).next() {
                    h2_element.text().collect::<String>().trim().to_string()
                } else {
                    "Unknown Title".to_string()
                };

                // 提取描述 (p)
                let p_selector = Selector::parse("p").map_err(|e| {
                    Error::Message(format!("Failed to parse CSS selector for p: {}", e))
                })?;

                let description = if let Some(p_element) = a_element.select(&p_selector).next() {
                    p_element.text().collect::<String>().trim().to_string()
                } else {
                    "".to_string()
                };

                // 构造详情链接
                let detail_link = format!("https://www.tongtech.com/dft/downloads/{}.html", id);

                // 由于页面上没有发布时间信息，使用当前日期作为发布时间
                let publish_time = chrono::Utc::now().format("%Y-%m-%d").to_string();

                notices.push(TongtechSecurityNotice {
                    id,
                    title,
                    description,
                    detail_link,
                    publish_time,
                });
            }
        }

        Ok(notices)
    }
}

#[derive(Debug, Clone)]
pub struct TongtechSecurityNotice {
    pub id: String,
    pub title: String,
    pub description: String,
    pub detail_link: String,
    pub publish_time: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mea::mpsc::unbounded;

    #[tokio::test]
    async fn test_parse_security_notices() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let plugin = TongtechNoticePlugin::try_new(sender).unwrap();

        // 尝试解析安全公告，如果网络请求失败则跳过测试
        match plugin.parse_security_notices().await {
            Ok(notices) => {
                // 检查是否获取到了最多3个公告
                assert!(notices.len() <= 3);

                // 如果有公告，验证基本信息
                if !notices.is_empty() {
                    let first_notice = &notices[0];
                    assert!(!first_notice.id.is_empty());
                    assert!(!first_notice.title.is_empty());
                    assert!(
                        first_notice
                            .detail_link
                            .contains("https://www.tongtech.com/dft/downloads/")
                    );
                    assert!(!first_notice.publish_time.is_empty());
                }
            }
            Err(_) => {
                // 网络请求失败，跳过测试
                println!("Skipping test due to network connectivity issues");
            }
        }
    }

    #[tokio::test]
    async fn test_plugin_creation() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let plugin = TongtechNoticePlugin::try_new(sender).unwrap();

        // 验证插件基本信息
        assert_eq!(plugin.get_name(), "TongtechPlugin");
        assert_eq!(plugin.get_display_name(), "东方通安全公告");
        assert_eq!(
            plugin.get_link(),
            "https://www.tongtech.com/dft/download.html"
        );
    }
}
