use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use mea::mpsc::UnboundedSender;
use regex::Regex;
use reqwest::Url;
use rquickjs::{Context, Runtime};
use scraper::{Html, Selector};
use serde_json::{Value, json};

use crate::{
    domain::models::vuln_information::{CreateVulnInformation, Severity},
    errors::Error,
    output::plugins::{VulnPlugin, register_plugin},
    utils::http_client::HttpClient,
};

const PAGE_REGEXP: &str = r"第 \d+ 页 / (\d+) 页 ";
const CVEID_REGEXP: &str = r"^CVE-\d+-\d+$";
const SCRIPT_REGEXP: &str = r#"(?s)<script[^>]*>(.*?)</script>"#;

#[derive(Debug, Clone)]
pub struct AVDPlugin {
    name: String,
    display_name: String,
    link: String,
    http_client: HttpClient,
    sender: UnboundedSender<CreateVulnInformation>,
}

#[async_trait]
impl VulnPlugin for AVDPlugin {
    fn get_name(&self) -> String {
        self.display_name.to_string()
    }
    async fn update(&self, page_limit: i32) -> Result<(), Error> {
        let mut page_count = self.get_page_count().await?;
        if page_count > page_limit {
            page_count = page_limit;
        }
        if let Some(i) = (1..=page_count).next() {
            let page_url = format!("{}?page={}", self.link, i);
            let document = self.get_document(&page_url).await?;
            let detail_links = self.get_detail_links(document)?;
            for detail in detail_links {
                let data = self.parse_detail_page(detail.as_ref()).await;
                match data {
                    Ok(data) => {
                        self.sender.send(data).change_context_lazy(|| {
                            Error::Message("Failed to send vuln information to channel".to_string())
                        })?;
                    }
                    Err(err) => log::error!("crawing detail {} error {}", detail, err),
                }
            }
        }
        Ok(())
    }
}

impl AVDPlugin {
    pub fn try_new(sender: UnboundedSender<CreateVulnInformation>) -> Result<AVDPlugin, Error> {
        let http_client = HttpClient::try_new()?;
        let avd = AVDPlugin {
            name: "AVDPlugin".to_string(),
            display_name: "阿里云漏洞库".to_string(),
            link: "https://avd.aliyun.com/high-risk/list".to_string(),
            http_client,
            sender,
        };
        register_plugin(avd.name.clone(), Box::new(avd.clone()));
        Ok(avd)
    }

    pub async fn get_page_count(&self) -> Result<i32, Error> {
        let new_url = self.waf_bypass(&self.link).await?;
        let content = self.http_client.get_html_content(&new_url).await?;

        let cap = Regex::new(PAGE_REGEXP)
            .change_context_lazy(|| Error::Message("page regex compile error".to_owned()))?
            .captures(&content);
        if let Some(res) = cap {
            if res.len() == 2 {
                let total = res[1]
                    .parse::<i32>()
                    .change_context_lazy(|| Error::Message("page parse error".to_owned()))?;
                Ok(total)
            } else {
                Err(Error::Message("page regex match error".to_owned()).into())
            }
        } else {
            Err(Error::Message("page regex match not found".to_owned()).into())
        }
    }

    pub async fn parse_page(&self, page: i32) -> Result<Vec<CreateVulnInformation>, Error> {
        let page_url = format!("{}?page={}", self.link, page);
        let document = self.get_document(&page_url).await?;
        let detail_links = self.get_detail_links(document)?;
        let mut res = Vec::with_capacity(detail_links.len());
        for detail in detail_links {
            let data = self.parse_detail_page(detail.as_ref()).await;
            match data {
                Ok(data) => res.push(data),
                Err(err) => log::warn!("crawing detail {} error {}", detail, err),
            }
        }
        Ok(res)
    }

    fn get_detail_links(&self, document: Html) -> Result<Vec<String>, Error> {
        let src_url_selector = Selector::parse("tbody tr td a")
            .map_err(|err| Error::Message(format!("selector parse error: {}", err)))?;

        let detail_links: Vec<String> = document
            .select(&src_url_selector)
            .filter_map(|a| a.value().attr("href"))
            .map(|l| l.to_string())
            .collect();
        Ok(detail_links)
    }

    pub async fn parse_detail_page(&self, href: &str) -> Result<CreateVulnInformation, Error> {
        let detail_url = format!("https://avd.aliyun.com{}", href);

        let document = self.get_document(&detail_url).await?;

        let avd_id = self.get_avd_id(&detail_url)?;

        let cve_id = self.get_cve_id(&document)?;
        if cve_id.is_empty() {
            log::warn!("cve id not found in {}", href);
        }

        let utilization = self.get_utilization(&document)?;
        let disclosure = self.get_disclosure(&document)?;
        let mut tags = Vec::new();
        if utilization != "暂无" {
            tags.push(utilization);
        }

        if cve_id.is_empty() && disclosure.is_empty() {
            return Err(Error::Message(format!("detail page not found in {}", href)).into());
        }

        let severity = self.get_severity(&document)?;

        let title = self.get_title(&document)?;

        let description = self.get_description(&document)?;

        let solutions = self.get_solutions(&document)?;

        let references = self.get_references(&document)?;

        let data = CreateVulnInformation {
            key: avd_id,
            title,
            description,
            severity: severity.to_string(),
            cve: cve_id,
            disclosure,
            reference_links: references,
            solutions,
            source: self.link.clone(),
            tags,
            reasons: vec![],
            github_search: vec![],
            pushed: false,
        };
        Ok(data)
    }

    fn get_avd_id(&self, detail_url: &str) -> Result<String, Error> {
        let url = Url::parse(detail_url)
            .map_err(|err| Error::Message(format!("avd get detail url parse error {}", err)))?;
        let avd_id = url
            .query_pairs()
            .filter(|(key, _)| key == "id")
            .map(|(_, value)| value)
            .collect::<Vec<_>>();
        let avd_id = avd_id[0].to_string();
        Ok(avd_id)
    }

    fn get_references(&self, document: &Html) -> Result<Vec<String>, Error> {
        let reference_selector = Selector::parse("td[nowrap='nowrap'] a").map_err(|err| {
            Error::Message(format!("avd get reference selector parse error {}", err))
        })?;
        let references = document
            .select(&reference_selector)
            .filter_map(|el| el.attr("href"))
            .map(|e| e.to_string())
            .collect::<Vec<_>>();
        Ok(references)
    }

    fn get_solutions(&self, document: &Html) -> Result<String, Error> {
        let solutions_selector = Selector::parse(".text-detail").map_err(|err| {
            Error::Message(format!("avd get solutions selector parse error {}", err))
        })?;
        let solutions = document
            .select(&solutions_selector)
            .nth(1)
            .ok_or_else(|| Error::Message("avd solutions value not found".to_string()))?
            .text()
            .map(|el| el.trim())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(solutions)
    }

    fn get_description(&self, document: &Html) -> Result<String, Error> {
        let description_selector = Selector::parse(".text-detail div").map_err(|err| {
            Error::Message(format!("avd get description selector parse error {}", err))
        })?;
        let description = document
            .select(&description_selector)
            .map(|e| e.text().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");
        Ok(description)
    }

    fn get_title(&self, document: &Html) -> Result<String, Error> {
        let title_selector = Selector::parse("h5[class='header__title'] .header__title__text")
            .map_err(|err| Error::Message(format!("avd get title selector parse error {}", err)))?;
        let title = document
            .select(&title_selector)
            .nth(0)
            .ok_or_else(|| Error::Message("avd title value not found".to_string()))?
            .inner_html()
            .trim()
            .to_string();
        Ok(title)
    }

    fn get_severity(&self, document: &Html) -> Result<Severity, Error> {
        let level_selector = Selector::parse("h5[class='header__title'] .badge")
            .map_err(|err| Error::Message(format!("avd get level selector parse error {}", err)))?;
        let level = document
            .select(&level_selector)
            .nth(0)
            .ok_or_else(|| Error::Message("avd level value not found".to_string()))?
            .inner_html()
            .trim()
            .to_string();
        let severity = match level.as_str() {
            "低危" => Severity::Low,
            "中危" => Severity::Medium,
            "高危" => Severity::High,
            "严重" => Severity::Critical,
            _ => Severity::Low,
        };
        Ok(severity)
    }

    fn get_mertric_value(&self, document: &Html, index: usize) -> Result<String, Error> {
        let value_selector = Selector::parse(".metric-value").map_err(|e| {
            Error::Message(format!("avd get metric value selector parse error {}", e))
        })?;
        let metric_value = document
            .select(&value_selector)
            .nth(index)
            .ok_or_else(|| Error::Message("avd get metric value not found".to_string()))?
            .inner_html()
            .trim()
            .to_string();
        Ok(metric_value)
    }

    fn get_cve_id(&self, document: &Html) -> Result<String, Error> {
        let mut cve_id = self.get_mertric_value(document, 0)?;
        if !Regex::new(CVEID_REGEXP)
            .change_context_lazy(|| Error::Message("avd get cve id regex parse error".to_string()))?
            .is_match(&cve_id)
        {
            cve_id = "".to_string();
        }
        Ok(cve_id)
    }

    fn get_utilization(&self, document: &Html) -> Result<String, Error> {
        self.get_mertric_value(document, 1)
    }

    fn get_disclosure(&self, document: &Html) -> Result<String, Error> {
        self.get_mertric_value(document, 3)
    }

    async fn get_document(&self, url: &str) -> Result<Html, Error> {
        let new_url = self.waf_bypass(url).await?;
        let content = self.http_client.get_html_content(&new_url).await?;
        let document = Html::parse_document(&content);
        Ok(document)
    }

    async fn waf_bypass(&self, target_url: &str) -> Result<String, Error> {
        let script_content = self.get_script_content(target_url).await?;
        if script_content.is_empty() {
            return Err(Error::Message("waf bypass script not found".to_string()).into());
        }
        let parsed_url = Url::parse(target_url)
            .map_err(|e| Error::Message(format!("waf bypass url parse error {}", e)))?;
        let location = json!({
            "href": target_url,
            "protocol": parsed_url.scheme().to_string() + ":",
            "host": parsed_url.host_str().unwrap_or(""),
            "hostname": parsed_url.host_str().unwrap_or(""),
            "port": parsed_url.port().map(|p| p.to_string()).unwrap_or_default(),
            "pathname": parsed_url.path(),
            "search": if parsed_url.query().is_some() {
                format!("?{}", parsed_url.query().unwrap())
            } else {
                String::new()
            },
            "hash": if !parsed_url.fragment().unwrap_or("").is_empty() {
                format!("#{}", parsed_url.fragment().unwrap())
            } else {
                String::new()
            }
        });
        let document = json!({
            "cookie": "",
            "location": location.clone()
        });

        let navigator = json!({
            "userAgent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
        });

        let window = json!({
            "navigator": navigator,
            "location": location.clone(),
            "document": document.clone()
        });
        let new_href = self
            .execute_javascript(&script_content, window, document, location.clone())
            .await?;
        Ok(new_href)
    }

    async fn execute_javascript(
        &self,
        script_content: &str,
        _window: Value,
        _document: Value,
        location: Value,
    ) -> Result<String, Error> {
        let runtime = Runtime::new()
            .map_err(|e| Error::Message(format!("execution script runtime new error {}", e)))?;
        let context = Context::full(&runtime)
            .map_err(|e| Error::Message(format!("execution script context full error {}", e)))?;
        context.with(|ctx| -> Result<String,Error> {
            let href = location["href"].as_str().unwrap_or("");
            let parsed_url = Url::parse(href)
                .map_err(|e| Error::Message(format!("url parsing error {}", e)))?;
            let url_parser_js = format!(r#"
                function urlParser() {{
                    return {{
                        protocol: "{}",
                        host: "{}",
                        hostname: "{}",
                        port: "{}",
                        pathname: "{}",
                        search: "{}",
                        hash: "{}",
                        url: "{}",
                        href: "{}",
                        firstChild: {{
                            protocol: "{}",
                            host: "{}",
                            hostname: "{}",
                            port: "{}",
                            pathname: "{}",
                            search: "{}",
                            hash: "{}",
                            url: "{}",
                            href: "{}"
                        }}
                    }};
                }}
            "#,
                parsed_url.scheme().to_string() + ":",
                parsed_url.host_str().unwrap_or(""),
                parsed_url.host_str().unwrap_or(""),
                parsed_url.port().map(|p| p.to_string()).unwrap_or_default(),
                parsed_url.path(),
                if parsed_url.query().is_some() { format!("?{}", parsed_url.query().unwrap()) } else { String::new() },
                if !parsed_url.fragment().unwrap_or("").is_empty() { format!("#{}", parsed_url.fragment().unwrap()) } else { String::new() },
                href, href,
                parsed_url.scheme().to_string() + ":",
                parsed_url.host_str().unwrap_or(""),
                parsed_url.host_str().unwrap_or(""),
                parsed_url.port().map(|p| p.to_string()).unwrap_or_default(),
                parsed_url.path(),
                if parsed_url.query().is_some() { format!("?{}", parsed_url.query().unwrap()) } else { String::new() },
                if !parsed_url.fragment().unwrap_or("").is_empty() { format!("#{}", parsed_url.fragment().unwrap()) } else { String::new() },
                href, href
            );

            let setup_js = format!(r#"
                {}
                // Create location object - must be mutable to detect changes
                var location = {{
                    href: "{}"
                }};

                // Create document with createElement returning urlParser() result
                var document = {{
                    cookie: "",
                    location: location,
                    createElement: function(args) {{
                        return urlParser();
                    }}
                }};

                // Create navigator
                var navigator = {{
                    userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
                }};

                // Create window object
                var window = {{
                    navigator: navigator,
                    location: location,
                    document: document
                }};

                // Add essential global functions
                var console = {{
                    log: function() {{}},
                    error: function() {{}},
                    warn: function() {{}},
                    info: function() {{}}
                }};

                var setTimeout = function(fn, delay) {{
                    if (typeof fn === 'function') {{
                        fn();
                    }}
                }};

                var setInterval = function(fn, delay) {{
                    if (typeof fn === 'function') {{
                        fn();
                    }}
                }};

                var btoa = function(str) {{
                    return "encoded_" + str;
                }};

                var atob = function(str) {{
                    return str.replace("encoded_", "");
                }};
            "#, url_parser_js, href);

            ctx.eval::<(), _>(setup_js).map_err(|e| Error::Message(format!("avd waf bypass setup js eval error {}", e)))?;

            if let Err(e) = ctx.eval::<(), _>(script_content) {
                return Err(Error::Message(format!("JavaScript execution error: {}", e)).into());
            }
            let new_target: String = ctx.eval("location.href").unwrap_or_else(|_| String::new());
            if new_target == href || new_target.is_empty() {
                Err(Error::Message("JavaScript did not modify location.href".to_string()).into())
            } else {
                Ok(new_target)
            }
        })
    }

    async fn get_script_content(&self, target_url: &str) -> Result<String, Error> {
        let origin_content = self.http_client.get_html_content(target_url).await?;
        let script_regex = Regex::new(SCRIPT_REGEXP)
            .map_err(|e| Error::Message(format!("avd get script regex parse error {}", e)))?;
        let script_content = script_regex
            .captures(&origin_content)
            .ok_or_else(|| Error::Message("avd waf bypass script not found".to_string()))?
            .get(1)
            .ok_or_else(|| Error::Message("avd waf bypass script capture not found".to_string()))?
            .as_str()
            .to_string();

        Ok(script_content)
    }
}
