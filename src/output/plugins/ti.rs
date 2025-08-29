use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use mea::mpsc::UnboundedSender;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::vuln_information::{CreateVulnInformation, Severity},
    errors::Error,
    output::plugins::{VulnPlugin, register_plugin},
    utils::http_client::HttpClient,
};

const ONE_URL: &str = "https://ti.qianxin.com/alpha-api/v2/vuln/one-day";

#[derive(Debug, Clone)]
pub struct TiPlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateVulnInformation>,
}

#[async_trait]
impl VulnPlugin for TiPlugin {
    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn get_display_name(&self) -> String {
        self.display_name.to_string()
    }

    fn get_link(&self) -> String {
        self.link.to_string()
    }

    async fn update(&self, _page_limit: i32) -> Result<(), Error> {
        self.get_vuln_infos().await
    }
}

impl TiPlugin {
    pub fn try_new(sender: UnboundedSender<CreateVulnInformation>) -> Result<TiPlugin, Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Referer",
            header::HeaderValue::from_static("https://ti.qianxin.com/"),
        );
        headers.insert(
            "Origin",
            header::HeaderValue::from_static("https://ti.qianxin.com/"),
        );

        let http_client = HttpClient::try_new_with_headers(headers)?;
        let ti = TiPlugin {
            name: "QiAnXinTiPlugin".to_string(),
            display_name: "奇安信威胁情报中心".to_string(),
            link: "https://ti.qianxin.com/".to_string(),
            http_client,
            sender,
        };
        register_plugin(ti.name.clone(), Box::new(ti.clone()));
        Ok(ti)
    }

    pub async fn get_vuln_infos(&self) -> Result<(), Error> {
        let ti_one_day_resp = self.get_ti_one_day_resp().await?;
        for detail in ti_one_day_resp.data.key_vuln_add {
            let tags = self.get_tags(detail.tag);
            let severity = self.get_severity(detail.rating_level);

            let data = CreateVulnInformation {
                key: detail.qvd_code,
                title: detail.vuln_name,
                description: detail.description,
                severity: severity.to_string(),
                cve: detail.cve_code,
                disclosure: detail.publish_time,
                reference_links: vec![],
                solutions: "".to_string(),
                source: self.link.to_string(),
                source_name: self.name.to_string(),
                tags,
                reasons: vec![],
                github_search: vec![],
                pushed: false,
            };
            self.sender.send(data).change_context_lazy(|| {
                Error::Message("Failed to send vuln information to channel".to_string())
            })?;
        }

        Ok(())
    }

    pub async fn get_ti_one_day_resp(&self) -> Result<TiOneDayResp, Error> {
        let params = serde_json::json!({});
        let resp: TiOneDayResp = self
            .http_client
            .post_json(ONE_URL, &params)
            .await?
            .json()
            .await
            .map_err(|e| Error::Message(format!("ti one day resp parse json failed: {e}")))?;
        Ok(resp)
    }

    pub fn get_tags(&self, detail_tags: Vec<Tag>) -> Vec<String> {
        let mut tags = Vec::with_capacity(detail_tags.len());
        for tag in detail_tags {
            tags.push(tag.name.trim().to_string());
        }
        tags
    }

    pub fn get_severity(&self, detail_severity: String) -> Severity {
        match detail_severity.as_str() {
            "低危" => Severity::Low,
            "中危" => Severity::Medium,
            "高危" => Severity::High,
            "极危" => Severity::Critical,
            _ => Severity::Low,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiOneDayResp {
    pub status: i32,
    pub message: String,
    pub data: Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub vuln_add_count: i32,
    pub vuln_update_count: i32,
    pub key_vuln_add_count: i32,
    pub poc_exp_add_count: i32,
    pub patch_add_count: i32,
    pub key_vuln_add: Vec<TiVulnDetail>,
    pub poc_exp_add: Vec<TiVulnDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiVulnDetail {
    pub id: i32,
    pub vuln_name: String,
    pub vuln_name_en: String,
    pub qvd_code: String,
    pub cve_code: String,
    pub cnvd_id: Option<String>,
    pub cnnvd_id: Option<String>,
    pub threat_category: String,
    pub technical_category: String,
    pub residence_id: Option<i32>,
    pub rating_id: Option<i32>,
    pub not_show: i32,
    pub publish_time: String,
    pub description: String,
    pub description_en: String,
    pub change_impact: i32,
    pub operator_hid: Option<String>,
    pub create_hid: Option<String>,
    pub channel: Option<String>,
    pub tracking_id: Option<String>,
    pub temp: i32,
    pub other_rating: i32,
    pub create_time: String,
    pub update_time: String,
    pub latest_update_time: String,
    pub rating_level: String,
    pub vuln_type: String,
    pub poc_flag: i32,
    pub patch_flag: i32,
    pub detail_flag: i32,
    pub tag: Vec<Tag>,
    pub tag_len: i32,
    pub is_rating_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub font_color: String,
    pub back_color: String,
}
