use async_trait::async_trait;
use mea::mpsc::UnboundedSender;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};

use crate::{
    AppResult,
    domain::models::security_notice::{CreateSecurityNotice, RiskLevel},
    errors::Error,
    output::plugins::sec_notice::{SecNoticePlugin, register_sec_notice},
    utils::http_client::HttpClient,
};

const PATCH_LINKS: &str = "https://www.smartbi.com.cn/patchinfo";

#[derive(Debug, Clone)]
pub struct SmartbiNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for SmartbiNoticePlugin {
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
        let patches = self.parse_security_patches().await?;
        for patch in patches {
            let create_security_notice = CreateSecurityNotice {
                key: self.generate_patch_key(&patch.date, &patch.description),
                title: patch.description,
                product_name: "Smartbi".to_string(),
                risk_level: RiskLevel::High.to_string(),
                source: self.link.clone(),
                source_name: self.get_name(),
                is_zero_day: false,
                publish_time: patch.date,
                detail_link: self.link.clone(),
                pushed: false,
            };
            self.sender.send(create_security_notice).map_err(|e| {
                Error::Message(format!("Failed to send security notice to queue: {}", e))
            })?;
        }
        Ok(())
    }
}

impl SmartbiNoticePlugin {
    pub fn try_new(
        sender: UnboundedSender<CreateSecurityNotice>,
    ) -> AppResult<SmartbiNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let smartbi = SmartbiNoticePlugin {
            name: "SmartbiPlugin".to_string(),
            display_name: "Smartbi安全补丁包".to_string(),
            link: PATCH_LINKS.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(smartbi.name.clone(), Box::new(smartbi.clone()));
        Ok(smartbi)
    }

    async fn get_document(&self, url: &str) -> AppResult<Html> {
        let content = self.http_client.get_html_content(url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    fn generate_patch_key(&self, date: &str, description: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(date.as_bytes());
        hasher.update(description.as_bytes());
        let result = hasher.finalize();

        format!("{:x}", result)
    }

    async fn parse_security_patches(&self) -> AppResult<Vec<SmartbiSecurityPatch>> {
        let document = self.get_document(&self.link).await?;

        // 查找包含"补丁更新记录"的div
        let selector = Selector::parse("div.i_cont")
            .map_err(|e| Error::Message(format!("Failed to parse CSS selector: {}", e)))?;
        let div_elements: Vec<_> = document.select(&selector).collect();

        let mut patches = Vec::new();

        for div in div_elements {
            let p_selector = Selector::parse("p")
                .map_err(|e| Error::Message(format!("Failed to parse CSS selector: {}", e)))?;
            let p_elements: Vec<_> = div.select(&p_selector).collect();

            for p in p_elements {
                let text = p.text().collect::<String>().trim().to_string();

                if text.is_empty() {
                    continue;
                }

                if let Some(patch) = self.parse_patch_text(&text) {
                    patches.push(patch);
                    // 补丁更新记录 并不会更新非常频繁，这里默认每次只获取前三条
                    if patches.len() >= 3 {
                        break;
                    }
                }
            }
            if patches.len() >= 3 {
                break;
            }
        }

        Ok(patches)
    }

    fn parse_patch_text(&self, text: &str) -> Option<SmartbiSecurityPatch> {
        let date_pattern = regex::Regex::new(r"\d{4}-\d{2}-\d{2}").ok()?;
        let date = date_pattern.find(text)?.as_str().to_string();

        let description = text[date_pattern.find(text)?.end()..].trim().to_string();

        if description.is_empty() {
            return None;
        }

        Some(SmartbiSecurityPatch { date, description })
    }
}

#[derive(Debug, Clone)]
pub struct SmartbiSecurityPatch {
    pub date: String,
    pub description: String,
}
