use async_trait::async_trait;
use error_stack::ResultExt;
use mea::mpsc::UnboundedSender;
use reqwest::header::{self, HeaderMap};
use serde::{Deserialize, Serialize};

use crate::{
    AppResult,
    domain::models::security_notice::{CreateSecurityNotice, RiskLevel},
    errors::Error,
    output::plugins::sec_notice::{SecNoticePlugin, register_sec_notice},
    utils::http_client::HttpClient,
};

const SANGFOR_API_URL: &str =
    "https://www.sangfor.com.cn/api/sf-os-document/openapi/dataManage/list";

#[derive(Debug, Clone)]
pub struct SangforNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for SangforNoticePlugin {
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
                product_name: "Sangfor".to_string(),
                risk_level: RiskLevel::High.to_string(),
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

impl SangforNoticePlugin {
    pub fn try_new(
        sender: UnboundedSender<CreateSecurityNotice>,
    ) -> AppResult<SangforNoticePlugin> {
        // 创建带有 site-id 头的 HTTP 客户端
        let mut headers = HeaderMap::new();
        headers.insert("site-id", header::HeaderValue::from_static("1684919726105"));
        let http_client = HttpClient::try_new_with_headers(headers)?;

        let sangfor = SangforNoticePlugin {
            name: "SangforPlugin".to_string(),
            display_name: "深信服安全公告".to_string(),
            link: "https://www.sangfor.com.cn".to_string(),
            http_client,
            sender,
        };
        register_sec_notice(sangfor.name.clone(), Box::new(sangfor.clone()));
        Ok(sangfor)
    }

    /// 解析安全公告信息
    ///
    /// 该方法会从深信服API获取安全公告列表，并提取前三条数据。
    ///
    /// # 返回值
    /// 返回解析到的安全公告列表或错误信息
    pub async fn parse_security_notices(&self) -> AppResult<Vec<SangforSecurityNotice>> {
        // 构造请求体
        let request_body = SangforApiRequest {
            module_type: "91ca821484911ecaac1fefcaa35fd11".to_string(),
            status_flow: 5,
            title: "".to_string(),
            year: "".to_string(),
            page_no: 1,
            page_size: 10,
        };

        // 发送 POST 请求
        let response = self
            .http_client
            .post_json(SANGFOR_API_URL, &request_body)
            .await
            .change_context_lazy(|| {
                Error::Message("Failed to send POST request to Sangfor API".to_string())
            })?;

        // 解析响应
        let response_text = response.text().await.change_context_lazy(|| {
            Error::Message("Failed to parse response body as text".to_string())
        })?;

        let api_response: SangforApiResponse = serde_json::from_str(&response_text)
            .change_context_lazy(|| {
                Error::Message("Failed to parse Sangfor API response".to_string())
            })?;

        if api_response.code != 0 {
            return Err(error_stack::Report::from(Error::Message(format!(
                "Sangfor API returned code:{} error: {}",
                api_response.code, api_response.msg
            ))));
        }

        // 处理前三条数据
        let mut notices = Vec::new();
        if let Some(data) = api_response.rows {
            for item in data.records.iter().take(3) {
                let notice = SangforSecurityNotice {
                    title: item.title.clone(),
                    detail_link: format!("https://www.sangfor.com.cn{}", item.url.clone()),
                    publish_time: item.publish_time.clone(),
                    unique_id: item.data_id.clone(),
                };
                notices.push(notice);
            }
        }

        Ok(notices)
    }
}

#[derive(Debug, Clone, Serialize)]
struct SangforApiRequest {
    #[serde(rename = "moduleType")]
    module_type: String,
    #[serde(rename = "statusFlow")]
    status_flow: i32,
    title: String,
    year: String,
    #[serde(rename = "pageNo")]
    page_no: i32,
    #[serde(rename = "pageSize")]
    page_size: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct SangforApiResponse {
    code: i32,
    msg: String,
    rows: Option<SangforApiData>,
}

#[derive(Debug, Clone, Deserialize)]
struct SangforApiData {
    records: Vec<SangforApiItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct SangforApiItem {
    #[serde(rename = "dataId")]
    data_id: String,
    title: String,
    url: String,
    #[serde(rename = "publishTime")]
    publish_time: String,
}

#[derive(Debug, Clone)]
pub struct SangforSecurityNotice {
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
        let plugin = SangforNoticePlugin::try_new(sender).unwrap();
        let notices = plugin.parse_security_notices().await.unwrap();
        // 检查是否获取到了最多3个公告
        assert!(notices.len() <= 3);
    }
}
