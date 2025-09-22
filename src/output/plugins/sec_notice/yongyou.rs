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

const YONGYOU_NOTICE_URL: &str = "https://security.yonyou.com/web-api/web/notice/page";
const YONGYOU_DEFAULT_PAGE_NO: i32 = 1;
const YONGYOU_DEFAULT_PAGE_SIZE: i32 = 3;

#[derive(Debug, Clone)]
pub struct YongYouNoticePlugin {
    name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for YongYouNoticePlugin {
    fn get_name(&self) -> String {
        self.name.to_string()
    }

    async fn update(&self, _page_limit: i32) -> AppResult<()> {
        let sec_notices = self
            .get_sec_notices(YONGYOU_DEFAULT_PAGE_NO, YONGYOU_DEFAULT_PAGE_SIZE)
            .await?;
        for sec_notice in sec_notices.data.list {
            let detail_link = format!(
                "https://security.yonyou.com/#/noticeInfo?id={}",
                sec_notice.id
            );
            let risk_level = self.get_risk_level(&sec_notice.risk_level).to_string();
            let create_security_notice = CreateSecurityNotice {
                key: sec_notice.identifier,
                title: sec_notice.notice,
                detail_link,
                source: self.link.clone(),
                source_name: self.get_name(),
                product_name: sec_notice.product_line_name,
                is_zero_day: self.is_zero_day(&sec_notice.is_zero_day),
                publish_time: sec_notice.publish_time,
                risk_level,
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

impl YongYouNoticePlugin {
    pub fn try_new(
        sender: UnboundedSender<CreateSecurityNotice>,
    ) -> AppResult<YongYouNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let yongyou = YongYouNoticePlugin {
            name: "YongYouPlugin".to_string(),
            link: "https://security.yonyou.com/#/home".to_string(),
            http_client,
            sender,
        };
        register_sec_notice(yongyou.name.clone(), Box::new(yongyou.clone()));
        Ok(yongyou)
    }

    pub async fn get_sec_notices(
        &self,
        page_no: i32,
        page_size: i32,
    ) -> AppResult<ListYongYouSecNotices> {
        let params = serde_json::json!({
            "pageNo": page_no,
            "pageSize": page_size,
        });
        let sec_notices: ListYongYouSecNotices = self
            .http_client
            .post_json(YONGYOU_NOTICE_URL, &params)
            .await?
            .json()
            .await
            .map_err(|e| Error::Message(format!("yongyou get sec notices error: {}", e)))?;
        Ok(sec_notices)
    }

    pub fn get_risk_level(&self, risk_level: &str) -> RiskLevel {
        match risk_level {
            "1" => RiskLevel::Critical,
            "2" => RiskLevel::High,
            "3" => RiskLevel::Medium,
            "4" => RiskLevel::Low,
            _ => RiskLevel::Low,
        }
    }

    pub fn is_zero_day(&self, zero_day: &str) -> bool {
        matches!(zero_day, "是")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListYongYouSecNotices {
    code: i32,
    data: YongYouNoticeData,
    msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YongYouNoticeData {
    list: Vec<YongYouNotice>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YongYouNotice {
    pub id: i32,
    pub notice: String,
    pub identifier: String,
    pub product_line_name: String,
    pub publish_time: String,
    pub risk_level: String,
    pub is_zero_day: String,
}
