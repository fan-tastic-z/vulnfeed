use async_trait::async_trait;
use error_stack::ResultExt;
use mea::mpsc::UnboundedSender;
use scraper::{ElementRef, Html, Selector};

use crate::{
    AppResult,
    domain::models::security_notice::{CreateSecurityNotice, RiskLevel},
    errors::Error,
    output::plugins::sec_notice::{SecNoticePlugin, register_sec_notice},
    utils::http_client::HttpClient,
};

const FIREFOX_NOTICE_URL: &str =
    "https://www.mozilla.org/en-US/security/known-vulnerabilities/firefox/";

#[derive(Debug, Clone)]
pub struct FirefoxNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for FirefoxNoticePlugin {
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
            // 获取详情页面信息
            let detail_info = self.get_detail_info(&notice.detail_link).await?;

            let create_security_notice = CreateSecurityNotice {
                key: notice.id,
                title: notice.title,
                product_name: detail_info.products,
                risk_level: self.map_risk_level(&detail_info.impact).to_string(),
                source: self.link.clone(),
                source_name: self.get_name(),
                is_zero_day: false, // Firefox公告通常不是零日漏洞
                publish_time: detail_info.announced,
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

impl FirefoxNoticePlugin {
    pub fn try_new(
        sender: UnboundedSender<CreateSecurityNotice>,
    ) -> AppResult<FirefoxNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let firefox = FirefoxNoticePlugin {
            name: "FirefoxPlugin".to_string(),
            display_name: "Firefox安全公告".to_string(),
            link: FIREFOX_NOTICE_URL.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(firefox.name.clone(), Box::new(firefox.clone()));
        Ok(firefox)
    }

    async fn get_document(&self, url: &str) -> AppResult<Html> {
        let content = self.http_client.get_html_content(url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    /// 解析主页获取安全公告列表
    pub async fn parse_security_notices(&self) -> AppResult<Vec<FirefoxSecurityNotice>> {
        let document = self.get_document(&self.link).await?;
        let article_selector = Selector::parse("article.mzp-c-article").map_err(|e| {
            Error::Message(format!("Failed to parse CSS selector for article: {}", e))
        })?;

        let article = document
            .select(&article_selector)
            .next()
            .ok_or_else(|| Error::Message("Failed to find article element".to_string()))?;

        // 选择除了header之外的所有h3元素（这些是版本标题）
        let h3_selector = Selector::parse("h3:not(header h3)")
            .map_err(|e| Error::Message(format!("Failed to parse CSS selector for h3: {}", e)))?;

        let mut notices = Vec::new();

        // 遍历所有h3元素，获取对应的漏洞信息
        for h3_element in article.select(&h3_selector) {
            // 检查是否已经有3个了
            if notices.len() >= 3 {
                break;
            }

            // 获取版本信息
            let version = h3_element.text().collect::<String>().trim().to_string();

            // 获取下一个兄弟元素
            let mut next_sibling = h3_element.next_sibling();
            while let Some(sibling) = next_sibling {
                if let Some(element) = sibling.value().as_element()
                    && element.name() == "ul"
                {
                    let ul_element = ElementRef::wrap(sibling)
                        .ok_or_else(|| Error::Message("Failed to wrap element".to_string()))?;

                    let li_selector = Selector::parse("li.level-item a").map_err(|e| {
                        Error::Message(format!("Failed to parse CSS selector for li: {}", e))
                    })?;

                    // 获取每个漏洞条目
                    for a_element in ul_element.select(&li_selector) {
                        if notices.len() >= 3 {
                            break;
                        }

                        let title = a_element.text().collect::<String>().trim().to_string();
                        let href = a_element.value().attr("href").unwrap_or("").to_string();
                        let id = href.split('/').nth_back(1).unwrap_or("").to_string();

                        if !href.is_empty() && !id.is_empty() {
                            notices.push(FirefoxSecurityNotice {
                                id,
                                title,
                                version: version.clone(),
                                detail_link: format!("https://www.mozilla.org{}", href),
                            });
                        }
                    }
                    break;
                }
                next_sibling = sibling.next_sibling();
            }
        }

        // 确保只返回前3个
        notices.truncate(3);
        Ok(notices)
    }

    /// 获取详情页面信息
    pub async fn get_detail_info(&self, url: &str) -> AppResult<FirefoxDetailInfo> {
        let document = self.get_document(url).await?;
        let article_selector = Selector::parse("article.mzp-c-article").map_err(|e| {
            Error::Message(format!("Failed to parse CSS selector for article: {}", e))
        })?;

        let article = document.select(&article_selector).next().ok_or_else(|| {
            Error::Message("Failed to find article element in detail page".to_string())
        })?;

        let mut detail_info = FirefoxDetailInfo::default();

        // 查找包含信息的dl元素
        let dl_selector = Selector::parse("dl.summary")
            .map_err(|e| Error::Message(format!("Failed to parse CSS selector for dl: {}", e)))?;

        if let Some(dl_element) = article.select(&dl_selector).next() {
            let dt_selector = Selector::parse("dt").map_err(|e| {
                Error::Message(format!("Failed to parse CSS selector for dt: {}", e))
            })?;
            let dd_selector = Selector::parse("dd").map_err(|e| {
                Error::Message(format!("Failed to parse CSS selector for dd: {}", e))
            })?;

            let dt_elements: Vec<_> = dl_element.select(&dt_selector).collect();
            let dd_elements: Vec<_> = dl_element.select(&dd_selector).collect();

            // 匹配dt和dd元素
            for (dt, dd) in dt_elements.iter().zip(dd_elements.iter()) {
                let dt_text = dt.text().collect::<String>().trim().to_string();
                let dd_text = dd.text().collect::<String>().trim().to_string();

                match dt_text.as_str() {
                    "Announced" => detail_info.announced = dd_text,
                    "Impact" => {
                        // Impact可能包含在span中
                        if let Ok(span_selector) = Selector::parse("span") {
                            if let Some(span_element) = dd.select(&span_selector).next() {
                                detail_info.impact =
                                    span_element.text().collect::<String>().trim().to_string();
                            } else {
                                detail_info.impact = dd_text;
                            }
                        } else {
                            detail_info.impact = dd_text;
                        }
                    }
                    "Products" => detail_info.products = dd_text,
                    "Fixed in" => {
                        // Fixed in 可能是一个列表
                        if let Ok(ul_selector) = Selector::parse("ul") {
                            if let Some(ul_element) = dd.select(&ul_selector).next() {
                                let li_selector = Selector::parse("li").map_err(|e| {
                                    Error::Message(format!(
                                        "Failed to parse CSS selector for li: {}",
                                        e
                                    ))
                                })?;
                                let versions: Vec<_> = ul_element
                                    .select(&li_selector)
                                    .map(|li| li.text().collect::<String>().trim().to_string())
                                    .collect();
                                detail_info.fixed_in = versions.join(", ");
                            } else {
                                detail_info.fixed_in = dd_text;
                            }
                        } else {
                            detail_info.fixed_in = dd_text;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(detail_info)
    }

    /// 将Firefox的风险等级映射到系统风险等级
    fn map_risk_level(&self, impact: &str) -> RiskLevel {
        match impact.to_lowercase().as_str() {
            "critical" => RiskLevel::Critical,
            "high" => RiskLevel::High,
            "moderate" => RiskLevel::Medium,
            "low" => RiskLevel::Low,
            _ => RiskLevel::Low,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FirefoxSecurityNotice {
    pub id: String,
    pub title: String,
    pub version: String,
    pub detail_link: String,
}

#[derive(Debug, Clone, Default)]
pub struct FirefoxDetailInfo {
    pub announced: String,
    pub impact: String,
    pub products: String,
    pub fixed_in: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mea::mpsc::unbounded;

    #[tokio::test]
    async fn test_parse_security_notices() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let plugin = FirefoxNoticePlugin::try_new(sender).unwrap();

        // 测试解析安全公告
        let notices = plugin.parse_security_notices().await.unwrap();

        // 验证是否获取到了公告
        assert!(!notices.is_empty());

        // 验证前3个公告
        assert!(notices.len() <= 3);

        // 验证第一个公告的基本信息
        let first_notice = &notices[0];
        assert!(!first_notice.id.is_empty());
        assert!(!first_notice.title.is_empty());
        assert!(!first_notice.version.is_empty());
        assert!(first_notice.detail_link.contains("https://www.mozilla.org"));

        // 验证详情页面信息获取
        let detail_info = plugin
            .get_detail_info(&first_notice.detail_link)
            .await
            .unwrap();
        assert!(!detail_info.announced.is_empty());
        assert!(!detail_info.impact.is_empty());
        assert!(!detail_info.products.is_empty());
        assert!(!detail_info.fixed_in.is_empty());
    }

    #[test]
    fn test_map_risk_level() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let plugin = FirefoxNoticePlugin::try_new(sender).unwrap();

        // 测试风险等级映射
        assert_eq!(plugin.map_risk_level("critical"), RiskLevel::Critical);
        assert_eq!(plugin.map_risk_level("high"), RiskLevel::High);
        assert_eq!(plugin.map_risk_level("moderate"), RiskLevel::Medium);
        assert_eq!(plugin.map_risk_level("low"), RiskLevel::Low);
        assert_eq!(plugin.map_risk_level("unknown"), RiskLevel::Low);
    }
}
