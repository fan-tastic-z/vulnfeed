use async_trait::async_trait;
use mea::mpsc::UnboundedSender;
use scraper::{ElementRef, Html, Selector};
use sha2::{Digest, Sha256};

use crate::{
    AppResult,
    domain::models::security_notice::{CreateSecurityNotice, RiskLevel},
    errors::Error,
    output::plugins::sec_notice::{SecNoticePlugin, register_sec_notice},
    utils::http_client::HttpClient,
};

const FANRUAN_NOTICE_URL: &str = "https://help.fanruan.com/finereport/doc-view-4833.html";

#[derive(Debug, Clone)]
pub struct FanRuanNoticePlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateSecurityNotice>,
}

#[async_trait]
impl SecNoticePlugin for FanRuanNoticePlugin {
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
                key: self.generate_patch_key(&notice.date, &notice.description),
                title: notice.description,
                product_name: "帆软".to_string(),
                risk_level: RiskLevel::High.to_string(),
                source: self.link.clone(),
                source_name: self.get_name(),
                is_zero_day: false,
                publish_time: notice.date,
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

impl FanRuanNoticePlugin {
    pub fn try_new(
        sender: UnboundedSender<CreateSecurityNotice>,
    ) -> AppResult<FanRuanNoticePlugin> {
        let http_client = HttpClient::try_new()?;
        let fanruan = FanRuanNoticePlugin {
            name: "FanRuanPlugin".to_string(),
            display_name: "帆软安全公告".to_string(),
            link: FANRUAN_NOTICE_URL.to_string(),
            http_client,
            sender,
        };
        register_sec_notice(fanruan.name.clone(), Box::new(fanruan.clone()));
        Ok(fanruan)
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

    async fn parse_security_notices(&self) -> AppResult<Vec<FanRuanSecurityNotice>> {
        let document = self.get_document(&self.link).await?;

        let h3_selector = Selector::parse("h3")
            .map_err(|e| Error::Message(format!("Failed to parse CSS selector: {}", e)))?;
        let h3_elements: Vec<_> = document.select(&h3_selector).collect();

        let target_h3 = h3_elements
            .iter()
            .find(|element| element.text().collect::<String>().contains("1.1 版本"));

        if target_h3.is_none() {
            return Ok(Vec::new());
        }

        let mut siblings = target_h3.unwrap().next_siblings();
        let table_element = siblings
            .find_map(|node| ElementRef::wrap(node).filter(|el| el.value().name() == "table"));

        if table_element.is_none() {
            return Ok(Vec::new());
        }

        let mut notices = Vec::new();
        let row_selector = Selector::parse("tr")
            .map_err(|e| Error::Message(format!("Failed to parse CSS selector: {}", e)))?;
        let rows: Vec<_> = table_element.unwrap().select(&row_selector).collect();

        for (_, row) in rows.iter().enumerate().skip(1) {
            let cell_selector = Selector::parse("td")
                .map_err(|e| Error::Message(format!("Failed to parse CSS selector: {}", e)))?;
            let cells: Vec<_> = row.select(&cell_selector).collect();

            if cells.len() >= 2 {
                let date = cells[0].text().collect::<String>().trim().to_string();
                let description = cells[1]
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string();
                notices.push(FanRuanSecurityNotice { date, description });

                if notices.len() >= 3 {
                    break;
                }
            }

            if notices.len() >= 3 {
                break;
            }
        }

        Ok(notices)
    }
}

#[derive(Debug, Clone)]
pub struct FanRuanSecurityNotice {
    pub date: String,
    pub description: String,
}
