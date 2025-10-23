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

const APPLE_NOTICE_URL: &str = "https://support.apple.com/zh-cn/100100";

#[derive(Debug, Clone)]
pub struct AppleNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for AppleNoticePlugin {
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
            // 获取详情页面信息
            let detail_info = self.get_detail_info(&notice.detail_link).await?;

            let create_security_notice = CreateSecurityNotice {
                key: notice.id,
                title: detail_info.title, // 使用详情页的产品名称作为标题
                product_name: notice.products, // 使用主页表格中的名称作为产品名称
                risk_level: RiskLevel::Critical.to_string(), // Apple公告通常不包含风险等级信息
                source: self.link.clone(),
                source_name: self.get_name(),
                is_zero_day: false,                  // Apple公告通常不是零日漏洞
                publish_time: notice.published_date, // 使用从主页表格中提取的发布日期
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

impl AppleNoticePlugin {
    pub fn try_new(sender: UnboundedSender<CreateSecurityNotice>) -> AppResult<AppleNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let apple = AppleNoticePlugin {
            name: "ApplePlugin".to_string(),
            display_name: "Apple安全公告".to_string(),
            link: APPLE_NOTICE_URL.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(apple.name.clone(), Box::new(apple.clone()));
        Ok(apple)
    }

    async fn get_document(&self, url: &str) -> AppResult<Html> {
        let content = self.http_client.get_html_content(url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    /// 解析主页获取安全公告列表
    pub async fn parse_security_notices(&self) -> AppResult<Vec<AppleSecurityNotice>> {
        let document = self.get_document(&self.link).await?;
        let table_selector = Selector::parse("div.table-wrapper.gb-table table").map_err(|e| {
            Error::Message(format!("Failed to parse CSS selector for table: {}", e))
        })?;

        let table = document
            .select(&table_selector)
            .next()
            .ok_or_else(|| Error::Message("Failed to find table element".to_string()))?;

        let tbody_selector = Selector::parse("tbody").map_err(|e| {
            Error::Message(format!("Failed to parse CSS selector for tbody: {}", e))
        })?;

        let tbody = table
            .select(&tbody_selector)
            .next()
            .ok_or_else(|| Error::Message("Failed to find tbody element".to_string()))?;

        let row_selector = Selector::parse("tr")
            .map_err(|e| Error::Message(format!("Failed to parse CSS selector for tr: {}", e)))?;

        let mut notices = Vec::new();

        // 跳过表头行，从第二行开始处理
        for row in tbody.select(&row_selector).skip(1) {
            if notices.len() >= 10 {
                break;
            }

            let cell_selector = Selector::parse("td").map_err(|e| {
                Error::Message(format!("Failed to parse CSS selector for td: {}", e))
            })?;

            let cells: Vec<_> = row.select(&cell_selector).collect();

            // 至少需要3个单元格（名称链接、适用产品、发布日期）
            if cells.len() >= 3 {
                // 提取发布日期
                let published_date = cells[2].text().collect::<String>().trim().to_string();

                // 提取名称和链接
                let name_cell = &cells[0];
                let anchor_selector = Selector::parse("a").map_err(|e| {
                    Error::Message(format!("Failed to parse CSS selector for a: {}", e))
                })?;

                if let Some(anchor) = name_cell.select(&anchor_selector).next() {
                    let products = anchor.text().collect::<String>().trim().to_string();
                    let href = anchor.value().attr("href").unwrap_or("").to_string();

                    if !href.is_empty() {
                        // 构造完整的详情链接
                        let detail_link = format!("https://support.apple.com{}", href);
                        // 从href中提取ID
                        let id = href.split('/').next_back().unwrap_or("").to_string();

                        notices.push(AppleSecurityNotice {
                            id,
                            products,
                            detail_link,
                            published_date,
                        });
                    }
                }
            }
        }

        Ok(notices)
    }

    /// 获取详情页面信息
    pub async fn get_detail_info(&self, url: &str) -> AppResult<AppleDetailInfo> {
        let document = self.get_document(url).await?;

        let mut detail_info = AppleDetailInfo::default();

        // 提取标题作为产品名称
        let header_selector = Selector::parse("h1.gb-header").map_err(|e| {
            Error::Message(format!(
                "Failed to parse CSS selector for h1.gb-header: {}",
                e
            ))
        })?;

        if let Some(header) = document.select(&header_selector).next() {
            detail_info.title = header.text().collect::<String>().trim().to_string();
        }

        Ok(detail_info)
    }
}

#[derive(Debug, Clone)]
pub struct AppleSecurityNotice {
    pub id: String,
    pub products: String,
    pub detail_link: String,
    pub published_date: String, // 添加发布日期字段
}

#[derive(Debug, Clone, Default)]
pub struct AppleDetailInfo {
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mea::mpsc::unbounded;

    #[tokio::test]
    async fn test_parse_security_notices() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let plugin = AppleNoticePlugin::try_new(sender).unwrap();

        // 测试解析安全公告
        let notices = plugin.parse_security_notices().await.unwrap();

        // 验证是否获取到了公告
        assert!(!notices.is_empty());

        // 验证前10个公告
        assert!(notices.len() <= 10);

        // 验证第一个公告的基本信息
        let first_notice = &notices[0];
        assert!(!first_notice.id.is_empty());
        assert!(!first_notice.products.is_empty());
        assert!(
            first_notice
                .detail_link
                .contains("https://support.apple.com")
        );

        // 验证详情页面信息获取
        let detail_info = plugin
            .get_detail_info(&first_notice.detail_link)
            .await
            .unwrap();

        assert!(!detail_info.title.is_empty());
    }
}
