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

const HOME_PAGE_URL: &str = "https://x.threatbook.com/v5/node/vul_module/homePage";
const LINK: &str = "https://x.threatbook.com/v5/vulIntelligence";

#[derive(Debug, Clone)]
pub struct ThreatBookPlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateVulnInformation>,
}

#[async_trait]
impl VulnPlugin for ThreatBookPlugin {
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
        let home_page_resp: ThreadBookHomePage = self
            .http_client
            .get_json(HOME_PAGE_URL)
            .await?
            .json()
            .await
            .map_err(|e| Error::Message(format!("Failed to parse home page response: {}", e)))?;
        for v in home_page_resp.data.high_risk {
            let disclosure = v.vuln_update_time.clone();

            let mut tags = Vec::new();
            if let Some(is_0day) = v.is_0day {
                if is_0day {
                    tags.push("0day".to_string());
                }
            }
            if v.poc_exist {
                tags.push("有Poc".to_string());
            }
            if v.premium {
                tags.push("有漏洞分析".to_string());
            }
            if v.solution {
                tags.push("有修复方案".to_string());
            }

            let data = CreateVulnInformation {
                key: v.id,
                title: v.vuln_name_zh,
                description: "".to_string(),
                severity: Severity::Critical.to_string(),
                cve: "".to_string(),
                disclosure,
                reference_links: Vec::new(),
                solutions: "".to_string(),
                source: LINK.to_string(),
                source_name: self.name.to_string(),
                tags,
                reasons: Vec::new(),
                github_search: vec![],
                pushed: false,
            };
            self.sender.send(data).change_context_lazy(|| {
                Error::Message("Failed to send vuln information to channel".to_string())
            })?;
        }
        Ok(())
    }
}

impl ThreatBookPlugin {
    pub fn try_new(
        sender: UnboundedSender<CreateVulnInformation>,
    ) -> Result<ThreatBookPlugin, Error> {
        let mut headers: reqwest::header::HeaderMap = header::HeaderMap::new();
        headers.insert("Referer", header::HeaderValue::from_static(LINK));
        headers.insert(
            "Origin",
            header::HeaderValue::from_static("https://mp.weixin.qq.com/"),
        );
        headers.insert(
            "Accept-Language",
            header::HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6"),
        );
        let http_client = HttpClient::try_new_with_headers(headers)?;
        let thread_book = ThreatBookPlugin {
            name: "ThreatBookPlugin".to_string(),
            display_name: "微步在线研究响应中心-漏洞通告".to_string(),
            link: LINK.to_string(),
            http_client,
            sender,
        };
        register_plugin(thread_book.name.clone(), Box::new(thread_book.clone()));
        Ok(thread_book)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadBookHomePage {
    pub data: Data,
    pub response_code: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "highrisk")]
    pub high_risk: Vec<HighRisk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighRisk {
    pub id: String,
    pub vuln_name_zh: String,
    pub vuln_update_time: String,
    pub affects: Vec<String>,
    pub vuln_publish_time: Option<String>,
    #[serde(rename = "pocExist")]
    pub poc_exist: bool,
    pub solution: bool,
    pub premium: bool,
    #[serde(rename = "riskLevel")]
    pub risk_level: String,
    #[serde(rename = "is0day")]
    pub is_0day: Option<bool>,
}
