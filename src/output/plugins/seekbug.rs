use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use mea::mpsc::UnboundedSender;
use scraper::{ElementRef, Html, Selector};

use crate::{
    domain::models::vuln_information::{CreateVulnInformation, Severity},
    errors::Error,
    output::plugins::{VulnPlugin, register_plugin},
    utils::http_client::HttpClient,
};

const SEEBUG_LIST_URL: &str = "https://www.seebug.org/vuldb/vulnerabilities";

#[derive(Debug, Clone)]
pub struct SeekBugPlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateVulnInformation>,
}

#[async_trait]
impl VulnPlugin for SeekBugPlugin {
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
        let mut page_count = self.get_page_count().await?;
        if page_count > page_limit {
            page_count = page_limit;
        }
        if let Some(i) = (1..=page_count).next() {
            self.parse_page(i).await?;
        }
        Ok(())
    }
}

impl SeekBugPlugin {
    pub fn try_new(sender: UnboundedSender<CreateVulnInformation>) -> Result<SeekBugPlugin, Error> {
        let http_client = HttpClient::try_new()?;
        let seebug = SeekBugPlugin {
            name: "SeeBugPlugin".to_string(),
            display_name: "Seebug 漏洞平台".to_string(),
            link: SEEBUG_LIST_URL.to_string(),
            http_client,
            sender,
        };
        register_plugin(seebug.name.clone(), Box::new(seebug.clone()));
        Ok(seebug)
    }

    pub async fn get_page_count(&self) -> Result<i32, Error> {
        let document = self.get_document(SEEBUG_LIST_URL).await?;
        let selector = Selector::parse("ul.pagination li a")
            .map_err(|err| Error::Message(format!("parse html error {}", err)))?;
        let page_nums = document
            .select(&selector)
            .map(|el| el.inner_html())
            .collect::<Vec<_>>();
        if page_nums.len() < 3 {
            return Err(Error::Message("failed to get seebug pagination node".to_owned()).into());
        }
        let total = page_nums[page_nums.len() - 1 - 1]
            .parse::<i32>()
            .map_err(|e| Error::Message(format!("parse total page error {}", e)))?;
        Ok(total)
    }

    pub async fn parse_page(&self, page: i32) -> Result<(), Error> {
        let url = format!("{}?page={}", SEEBUG_LIST_URL, page);
        let document = self.get_document(&url).await?;
        let selector = Selector::parse(".sebug-table tbody tr")
            .map_err(|err| Error::Message(format!("seebug parse html error {}", err)))?;
        let tr_elements = document.select(&selector).collect::<Vec<_>>();
        if tr_elements.is_empty() {
            return Err(Error::Message("failed to get seebug page".into()).into());
        }
        for el in tr_elements {
            let (href, unique_key) = match self.get_href(el) {
                Ok((href, unique_key)) => (href, unique_key),
                Err(e) => {
                    log::warn!("seebug get href error {}", e);
                    continue;
                }
            };
            let disclosure = match self.get_disclosure(el) {
                Ok(disclosure) => disclosure,
                Err(e) => {
                    log::warn!("seebug get disclosure error {}", e);
                    continue;
                }
            };
            let severity_title = match self.get_severity_title(el) {
                Ok(severity_title) => severity_title,
                Err(e) => {
                    log::warn!("seebug get severity title error {}", e);
                    continue;
                }
            };
            let title = match self.get_title(el) {
                Ok(title) => title,
                Err(e) => {
                    log::warn!("seebug get title error {}", e);
                    continue;
                }
            };
            let cve_id = match self.get_cve_id(el) {
                Ok(cve_id) => cve_id,
                Err(e) => {
                    log::warn!("seebug get cve_id error {}", e);
                    "".to_string()
                }
            };
            let tag = match self.get_tag(el) {
                Ok(tag) => tag,
                Err(e) => {
                    log::warn!("seebug get tag error {}", e);
                    "".to_string()
                }
            };

            let severity = self.get_severity(&severity_title);
            let mut tags = Vec::new();
            if !tag.is_empty() {
                tags.push(tag)
            }

            let data = CreateVulnInformation {
                key: unique_key,
                title,
                description: "".to_owned(),
                severity: severity.to_string(),
                cve: cve_id,
                disclosure,
                reference_links: vec![],
                solutions: "".to_owned(),
                source: href,
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

    fn get_severity(&self, severity_title: &str) -> Severity {
        match severity_title {
            "低危" => Severity::Low,
            "中危" => Severity::Medium,
            "高危" => Severity::High,
            _ => Severity::Low,
        }
    }

    async fn get_document(&self, url: &str) -> Result<Html, Error> {
        let content = self.http_client.get_html_content(url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    fn get_href(&self, el: ElementRef) -> Result<(String, String), Error> {
        let selector = Selector::parse("td a")
            .map_err(|err| Error::Message(format!("parse html error {}", err)))?;
        let a_element = el
            .select(&selector)
            .nth(0)
            .ok_or_else(|| Error::Message("value not found".to_string()))?;
        let href = a_element
            .value()
            .attr("href")
            .ok_or_else(|| Error::Message("href not found".to_string()))?
            .trim();
        let href = format!("https://www.seebug.org{}", href);
        let binding = a_element.inner_html();
        let unique_key = binding.trim();
        Ok((href.to_owned(), unique_key.to_owned()))
    }

    fn get_disclosure(&self, el: ElementRef) -> Result<String, Error> {
        let selector = Selector::parse("td")
            .map_err(|err| Error::Message(format!("parse html error {}", err)))?;
        let disclosure = el
            .select(&selector)
            .nth(1)
            .ok_or_else(|| Error::Message("value not found".to_string()))?
            .inner_html();

        Ok(disclosure)
    }

    fn get_severity_title(&self, el: ElementRef) -> Result<String, Error> {
        let selector = Selector::parse("td div")
            .map_err(|err| Error::Message(format!("parse html error {}", err)))?;
        let td_element = el
            .select(&selector)
            .nth(0)
            .ok_or_else(|| Error::Message("severity_title div not found".to_string()))?;
        let severity_title = td_element
            .value()
            .attr("data-original-title")
            .ok_or_else(|| Error::Message("href not found".to_string()))?
            .trim();
        Ok(severity_title.to_owned())
    }

    fn get_title(&self, el: ElementRef) -> Result<String, Error> {
        let selector = Selector::parse("td a[class='vul-title']")
            .map_err(|err| Error::Message(format!("parse html error {}", err)))?;
        let title = el
            .select(&selector)
            .nth(0)
            .ok_or_else(|| Error::Message("title not found".to_string()))?
            .inner_html();
        Ok(title)
    }

    fn get_cve_id(&self, el: ElementRef) -> Result<String, Error> {
        let selector = Selector::parse("td i[class='fa fa-id-card ']")
            .map_err(|err| Error::Message(format!("parse html error {}", err)))?;
        let cve_ids = el
            .select(&selector)
            .nth(0)
            .ok_or_else(|| Error::Message("cve id element not found".to_string()))?
            .value()
            .attr("data-original-title")
            .ok_or_else(|| Error::Message("data-original-title not found".to_string()))?
            .trim();
        if cve_ids.contains('、') {
            return Ok(cve_ids
                .split('、')
                .nth(0)
                .ok_or_else(|| Error::Message("cve_ids split not found cve id".to_string()))?
                .to_owned());
        }
        Ok(cve_ids.to_string())
    }

    fn get_tag(&self, el: ElementRef) -> Result<String, Error> {
        let selector = Selector::parse("td .fa-file-text-o")
            .map_err(|err| Error::Message(format!("parse html error {}", err)))?;
        let tag = el
            .select(&selector)
            .nth(0)
            .ok_or_else(|| Error::Message("tag not found".to_string()))?
            .value()
            .attr("data-original-title")
            .ok_or_else(|| Error::Message("data-original-title not found".to_string()))?
            .trim();
        Ok(tag.to_string())
    }
}
