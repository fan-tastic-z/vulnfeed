use async_trait::async_trait;
use error_stack::ResultExt;
use mea::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};

use crate::{
    AppResult,
    domain::models::security_notice::{CreateSecurityNotice, RiskLevel},
    errors::Error,
    output::plugins::sec_notice::{SecNoticePlugin, register_sec_notice},
    utils::http_client::HttpClient,
};

const SEEOYN_NOTICE_URL: &str = "https://service.seeyon.com/server2/rest/security/getCategory";
const SEEOYN_DETAIL_URL: &str =
    "https://service.seeyon.com/server2/rest/security/getCategoryDetail";

#[derive(Debug, Clone)]
pub struct SeeyonNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for SeeyonNoticePlugin {
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
            let title = notice.title.clone();

            let detail_response = self.get_detail_data(notice.id).await?;
            let (patch_number, patch_time) = self.extract_patch_info(&detail_response.content);
            let detail_link = format!("{}?id={}", SEEOYN_DETAIL_URL, notice.id);
            match (patch_number, patch_time) {
                (Some(patch_number), Some(patch_time)) => {
                    let create_security_notice = CreateSecurityNotice {
                        key: patch_number,
                        title,
                        product_name: "致远OA".to_string(),
                        risk_level: RiskLevel::High.to_string(),
                        source: self.link.clone(),
                        source_name: self.get_name(),
                        is_zero_day: false,
                        detail_link,
                        publish_time: patch_time,
                        pushed: false,
                    };
                    self.sender.send(create_security_notice).map_err(|e| {
                        Error::Message(format!("Failed to send security notice to queue: {}", e))
                    })?;
                }
                (_, _) => {
                    log::error!(
                        "Failed to extract patch info for notice id: {} title: {}",
                        notice.id,
                        notice.title
                    );
                    continue;
                }
            }
        }
        Ok(())
    }
}

impl SeeyonNoticePlugin {
    pub fn try_new(sender: UnboundedSender<CreateSecurityNotice>) -> AppResult<SeeyonNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let seeyon = SeeyonNoticePlugin {
            name: "SeeyonPlugin".to_string(),
            display_name: "致远OA安全公告".to_string(),
            link: "https://service.seeyon.com".to_string(),
            http_client,
            sender,
        };
        register_sec_notice(seeyon.name.clone(), Box::new(seeyon.clone()));
        Ok(seeyon)
    }

    async fn get_category_data(&self) -> AppResult<SeeyonCategoryResponse> {
        let response = self
            .http_client
            .get_json(SEEOYN_NOTICE_URL)
            .await
            .change_context_lazy(|| {
                Error::Message("Failed to fetch Seeyon category data".to_string())
            })?;
        let category_data: SeeyonCategoryResponse =
            response.json().await.change_context_lazy(|| {
                Error::Message("Failed to parse Seeyon category data".to_string())
            })?;
        Ok(category_data)
    }

    pub async fn get_detail_data(&self, id: i32) -> AppResult<SeeyonDetailResponse> {
        let url = format!("{}?id={}", SEEOYN_DETAIL_URL, id);
        let response = self
            .http_client
            .get_json(&url)
            .await
            .change_context_lazy(|| {
                Error::Message(format!("Failed to fetch Seeyon detail data for id: {}", id))
            })?;
        let detail_data: SeeyonDetailResponse =
            response.json().await.change_context_lazy(|| {
                Error::Message(format!("Failed to parse Seeyon detail data for id: {}", id))
            })?;
        Ok(detail_data)
    }

    async fn parse_security_notices(&self) -> AppResult<Vec<SeeyonSecurityNotice>> {
        let category_data = self.get_category_data().await?;

        let v5_id = category_data
            .iter()
            .find(|item| item.classification == "V5")
            .map(|item| item.id)
            .ok_or_else(|| Error::Message("V5 classification not found".to_string()))?;

        // Find items with pid equal to V5 id
        let mut notices: Vec<SeeyonSecurityNotice> = Vec::new();

        // 获取前3个ID最大的项目
        let mut filtered_items: Vec<_> = category_data
            .iter()
            .filter(|item| item.pid == v5_id)
            .collect();

        filtered_items.sort_by(|a, b| b.id.cmp(&a.id));
        filtered_items.truncate(3);

        // 为每个项目获取详细信息
        for item in filtered_items {
            let mut notice = SeeyonSecurityNotice {
                id: item.id,
                title: item.classification.clone(),
                patch_number: None,
                publish_time: None,
            };

            // 获取详细信息
            match self.get_detail_data(item.id).await {
                Ok(detail) => {
                    notice.title = detail.title;

                    // 从HTML内容中提取补丁编号和发布时间
                    let (patch_number, publish_time) = self.extract_patch_info(&detail.content);
                    notice.patch_number = patch_number;
                    notice.publish_time = publish_time;
                }
                Err(e) => {
                    eprintln!("Failed to get detail data for id {}: {:?}", item.id, e);
                }
            }
            notices.push(notice);
        }

        Ok(notices)
    }

    pub fn extract_patch_info(&self, content: &str) -> (Option<String>, Option<String>) {
        let mut patch_number = None;
        let mut publish_time = None;
        let document = scraper::Html::parse_document(content);

        // CSS选择器查找包含"补丁编号"文本的td元素，然后获取下一个兄弟元素的文本内容
        let patch_selector = scraper::Selector::parse("td").unwrap();
        let elements: Vec<_> = document.select(&patch_selector).collect();

        for (i, element) in elements.iter().enumerate() {
            if let Some(text) = element.text().next()
                && text.contains("补丁编号")
                && i + 1 < elements.len()
            {
                // 获取下一个td元素作为补丁编号
                patch_number = Some(
                    elements[i + 1]
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string(),
                );
                break;
            }
        }

        // CSS选择器查找包含"发布时间"文本的td元素，然后获取下一个兄弟元素的文本内容
        for (i, element) in elements.iter().enumerate() {
            if let Some(text) = element.text().next()
                && text.contains("发布时间")
                && i + 1 < elements.len()
            {
                // 获取下一个td元素作为发布时间
                publish_time = Some(
                    elements[i + 1]
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string(),
                );
                break;
            }
        }

        (patch_number, publish_time)
    }
}

pub type SeeyonCategoryResponse = Vec<SeeyonCategoryItem>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeeyonCategoryItem {
    pub id: i32,
    pub pid: i32,
    pub classification: String,
    pub sort: i32,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeeyonSecurityNotice {
    pub id: i32,
    pub title: String,
    pub patch_number: Option<String>,
    pub publish_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeeyonDetailResponse {
    pub id: i32,
    pub title: String,
    pub content: String,
    #[serde(rename = "createTime")]
    pub create_time: Option<i64>,
    pub author: String,
    #[serde(rename = "categoryId")]
    pub category_id: i32,
    #[serde(rename = "categoryName")]
    pub category_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mea::mpsc::unbounded;

    #[tokio::test]
    pub async fn test_extract_patch_info() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let plugin = SeeyonNoticePlugin::try_new(sender).unwrap();
        let detail_data = plugin.get_detail_data(180).await.unwrap();
        let content = &detail_data.content;

        let (patch_number, publish_time) = plugin.extract_patch_info(content);

        assert_eq!(patch_number, Some("250300-S006".to_string()));
        assert_eq!(publish_time, Some("2025年3月".to_string()));
    }
}
