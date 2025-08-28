use async_trait::async_trait;
use chrono::{DateTime, Utc};
use error_stack::{Result, ResultExt};
use mea::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::vuln_information::{CreateVulnInformation, Severity},
    errors::Error,
    output::plugins::{VulnPlugin, register_plugin},
    utils::http_client::HttpClient,
};

const KEV_URL: &str =
    "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json";
const KEV_PAGE_SIZE: usize = 10;

#[derive(Debug, Clone)]
pub struct KevPlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateVulnInformation>,
}

impl KevPlugin {
    pub fn try_new(sender: UnboundedSender<CreateVulnInformation>) -> Result<KevPlugin, Error> {
        let http_client = HttpClient::try_new()?;
        let kv = KevPlugin {
            name: "KevPlugin".to_string(),
            display_name: "Known Exploited Vulnerabilities Catalog".to_string(),
            link: "https://www.cisa.gov/known-exploited-vulnerabilities-catalog".to_string(),
            http_client,
            sender,
        };
        register_plugin(kv.name.clone(), Box::new(kv.clone()));
        Ok(kv)
    }
}

#[async_trait]
impl VulnPlugin for KevPlugin {
    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn get_display_name(&self) -> String {
        self.display_name.to_string()
    }

    fn get_link(&self) -> String {
        self.link.to_string()
    }

    async fn update(&self, page_limit: i32) -> Result<(), Error> {
        let kev_list_resp: KevResp = self
            .http_client
            .get_json(KEV_URL)
            .await?
            .json()
            .await
            .map_err(|e| {
                Error::Message(format!("Failed to parse KEV response json object {}", e))
            })?;
        let all_count = kev_list_resp.vulnerabilities.len();
        let item_limit = if page_limit as usize * KEV_PAGE_SIZE > all_count {
            all_count
        } else {
            page_limit as usize * KEV_PAGE_SIZE
        };
        let mut vulnerabilities = kev_list_resp.vulnerabilities;
        vulnerabilities.sort_by(|a, b| b.date_added.cmp(&a.date_added));
        for vuln in vulnerabilities.iter().take(item_limit) {
            let mut reference_links = Vec::new();
            if !vuln.notes.is_empty() {
                reference_links.push(vuln.notes.to_string())
            }
            let create_vuln_information_req = CreateVulnInformation {
                key: format!("{}_KEV", vuln.cve_id),
                title: vuln.vulnerability_name.to_string(),
                description: vuln.short_description.to_string(),
                severity: Severity::Critical.to_string(),
                cve: vuln.cve_id.to_string(),
                disclosure: vuln.date_added.to_string(),
                reference_links,
                solutions: vuln.required_action.to_string(),
                source: self.link.to_string(),
                source_name: self.name.to_string(),
                tags: vec![
                    vuln.vendor_project.to_string(),
                    vuln.product.to_string(),
                    "在野利用".to_string(),
                ],
                github_search: vec![],
                reasons: vec![],
                pushed: false,
            };
            self.sender
                .send(create_vuln_information_req)
                .change_context_lazy(|| {
                    Error::Message("Failed to send vuln information to channel".to_string())
                })?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct KevResp {
    pub title: String,
    pub catalog_version: String,
    pub date_released: DateTime<Utc>,
    pub count: i32,
    pub vulnerabilities: Vec<Vulnerability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Vulnerability {
    #[serde(alias = "cveID")]
    pub cve_id: String,
    pub vendor_project: String,
    pub product: String,
    pub vulnerability_name: String,
    pub date_added: String,
    pub short_description: String,
    pub required_action: String,
    pub due_date: String,
    pub known_ransomware_campaign_use: String,
    pub notes: String,
}
