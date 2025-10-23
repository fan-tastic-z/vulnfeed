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

const GRAFANA_SECURITY_URL: &str = "https://grafana.com/tags/security/";

#[derive(Debug, Clone)]
pub struct GrafanaNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for GrafanaNoticePlugin {
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

        // 只取前10条数据
        for notice in notices.into_iter().take(10) {
            let create_security_notice = CreateSecurityNotice {
                key: notice.id,
                title: notice.title,
                product_name: "Grafana".to_string(),
                risk_level: RiskLevel::Critical.to_string(), // Grafana安全公告通常为高风险
                source: self.link.clone(),
                source_name: self.get_name(),
                is_zero_day: false,
                publish_time: notice.published_date,
                detail_link: notice.detail_link,
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

impl GrafanaNoticePlugin {
    pub fn try_new(
        sender: UnboundedSender<CreateSecurityNotice>,
    ) -> AppResult<GrafanaNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let grafana = GrafanaNoticePlugin {
            name: "GrafanaPlugin".to_string(),
            display_name: "Grafana安全公告".to_string(),
            link: GRAFANA_SECURITY_URL.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(grafana.name.clone(), Box::new(grafana.clone()));
        Ok(grafana)
    }

    async fn get_document(&self, url: &str) -> AppResult<Html> {
        let content = self.http_client.get_html_content(url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    /// 解析安全公告列表
    pub async fn parse_security_notices(&self) -> AppResult<Vec<GrafanaSecurityNotice>> {
        let document = self.get_document(&self.link).await?;

        // 选择 class="all-posts" 下的所有 article 元素
        let articles_selector = Selector::parse(".all-posts article").map_err(|e| {
            Error::Message(format!("Failed to parse CSS selector for articles: {}", e))
        })?;

        let articles: Vec<_> = document.select(&articles_selector).collect();

        let mut notices = Vec::new();

        for article in articles.iter() {
            if notices.len() >= 10 {
                break;
            }

            // 提取标题
            let title_selector = Selector::parse("h3").map_err(|e| {
                Error::Message(format!("Failed to parse CSS selector for title: {}", e))
            })?;

            if let Some(title_element) = article.select(&title_selector).next() {
                let title = title_element.text().collect::<String>().trim().to_string();

                // 只处理以 "Grafana security release" 或 "Grafana security update" 开头的公告
                if title.starts_with("Grafana security release")
                    || title.starts_with("Grafana security update")
                {
                    // 提取详情链接
                    let link_selector = Selector::parse("a").map_err(|e| {
                        Error::Message(format!("Failed to parse CSS selector for link: {}", e))
                    })?;

                    let detail_link =
                        if let Some(link_element) = article.select(&link_selector).next() {
                            link_element
                                .value()
                                .attr("href")
                                .map(|href| {
                                    if href.starts_with("http") {
                                        href.to_string()
                                    } else {
                                        format!("https://grafana.com{}", href)
                                    }
                                })
                                .unwrap_or_default()
                        } else {
                            "".to_string()
                        };

                    // 提取发布日期
                    let date_selector =
                        Selector::parse("p.blog-list-item__byline").map_err(|e| {
                            Error::Message(format!("Failed to parse CSS selector for date: {}", e))
                        })?;

                    let published_date =
                        if let Some(date_element) = article.select(&date_selector).next() {
                            // 从 byline 中提取日期，格式类似 "Naima Alexander · 18 Sep 2025 · 5 min read"
                            let text = date_element.text().collect::<String>();
                            let parts: Vec<&str> = text.split("·").collect();
                            if parts.len() >= 2 {
                                parts[1].trim().to_string()
                            } else {
                                text.trim().to_string()
                            }
                        } else {
                            "".to_string()
                        };

                    // 从链接中提取ID
                    let id = detail_link
                        .split('/')
                        .filter(|s| !s.is_empty())
                        .next_back()
                        .unwrap_or(&detail_link)
                        .to_string();

                    notices.push(GrafanaSecurityNotice {
                        id,
                        title,
                        detail_link,
                        published_date,
                    });
                }
            }
        }

        Ok(notices)
    }
}

#[derive(Debug, Clone)]
pub struct GrafanaSecurityNotice {
    pub id: String,
    pub title: String,
    pub detail_link: String,
    pub published_date: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mea::mpsc::unbounded;

    #[tokio::test]
    async fn test_parse_security_notices() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let plugin = GrafanaNoticePlugin::try_new(sender).unwrap();

        // 测试解析安全公告
        let result = plugin.parse_security_notices().await;
        if let Err(e) = &result {
            println!("Error parsing security notices: {:?}", e);
        }
        let notices = result.unwrap();

        for (i, notice) in notices.iter().enumerate() {
            println!(
                "Notice {}: title={}, id={}, link={}",
                i, notice.title, notice.id, notice.detail_link
            );
        }

        // 验证是否获取到了公告
        // 验证是否获取到了公告
        assert!(!notices.is_empty());

        // 验证前10个公告
        assert!(notices.len() <= 10);

        // 验证每个公告的标题都以指定前缀开头
        for notice in &notices {
            assert!(
                notice.title.starts_with("Grafana security release")
                    || notice.title.starts_with("Grafana security update")
            );
            assert!(!notice.id.is_empty());
            assert!(!notice.detail_link.is_empty());
        }
    }
}
