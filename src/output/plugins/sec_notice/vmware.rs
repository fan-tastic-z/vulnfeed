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

const LIST_API: &str = "https://support.broadcom.com/web/ecx/security-advisory/-/securityadvisory/getSecurityAdvisoryList";
const PAGE: &str = "https://support.broadcom.com/web/ecx/security-advisory?";
const DEFAULT_PAGE_SIZE: usize = 10;

#[derive(Debug, Clone)]
pub struct VmwareNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for VmwareNoticePlugin {
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
        let resp = self
            .get_security_advisory_list(0, DEFAULT_PAGE_SIZE)
            .await?;
        if resp.success {
            for item in resp.data.list {
                let create_security_notice = CreateSecurityNotice {
                    key: item.document_id,
                    title: item.title,
                    product_name: item.support_products,
                    risk_level: self.get_risk_level(&item.severity).to_string(),
                    source: self.link.clone(),
                    source_name: self.get_name(),
                    is_zero_day: true, // 这里默认都是true
                    publish_time: item.updated,
                    detail_link: item.notification_url,
                    pushed: false,
                };
                self.sender
                    .send(create_security_notice)
                    .change_context_lazy(|| {
                        Error::Message("Failed to send security notice to queue".to_string())
                    })?;
            }
        }
        Ok(())
    }
}

impl VmwareNoticePlugin {
    pub fn try_new(sender: UnboundedSender<CreateSecurityNotice>) -> AppResult<VmwareNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let vmware = VmwareNoticePlugin {
            name: "VmwarePlugin".to_string(),
            display_name: "Vmware 安全公告".to_string(),
            link: PAGE.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(vmware.name.clone(), Box::new(vmware.clone()));
        Ok(vmware)
    }

    pub fn get_risk_level(&self, severity: &str) -> RiskLevel {
        match severity {
            "CRITICAL" => RiskLevel::Critical,
            "HIGH" => RiskLevel::High,
            "MEDIUM" => RiskLevel::Medium,
            "LOW" => RiskLevel::Low,
            _ => RiskLevel::Critical,
        }
    }

    pub async fn get_security_advisory_list(
        &self,
        page_number: usize,
        page_size: usize,
    ) -> AppResult<SecurityAdvisoryListResponse> {
        let params = serde_json::json!({
            "pageNumber": page_number,
            "pageSize": page_size,
        });
        let security_advisory_list_response: SecurityAdvisoryListResponse = self
            .http_client
            .post_json(LIST_API, &params)
            .await?
            .json()
            .await
            .map_err(|e| Error::Message(format!("vmware get list resp error: {}", e)))?;
        Ok(security_advisory_list_response)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAdvisoryListResponse {
    pub success: bool,
    pub data: SecurityAdvisoryList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAdvisoryList {
    pub list: Vec<SecurityAdvisory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAdvisory {
    pub document_id: String,
    pub notification_id: i32,
    pub published: String,
    pub title: String,
    pub severity: String,
    pub updated: String,
    pub notification_url: String,
    pub support_products: String,
}

#[cfg(test)]
mod tests {
    use mea::mpsc::unbounded;

    use super::*;

    #[tokio::test]
    pub async fn test_get_security_advisory_list() {
        let (sender, _receiver) = unbounded::<CreateSecurityNotice>();
        let res = VmwareNoticePlugin::try_new(sender).unwrap();
        let list = res
            .get_security_advisory_list(0, 10)
            .await
            .unwrap()
            .data
            .list;
        assert!(!list.is_empty());
        assert_eq!(list.len(), 10);
    }
}
