use error_stack::{Result, ResultExt};

use crate::errors::Error;

#[derive(Debug, Clone)]
pub struct HttpClient {
    http_client: reqwest::Client,
}

impl HttpClient {
    pub fn try_new() -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .danger_accept_invalid_certs(true)
            .build()
            .change_context_lazy(|| Error::Message("Failed to create HTTP client".to_string()))?;
        Ok(Self {
            http_client: client,
        })
    }

    pub async fn get_json(&self, url: &str) -> Result<reqwest::Response, Error> {
        let content = self
            .http_client
            .get(url)
            .send()
            .await
            .change_context_lazy(|| Error::Message("Failed to send HTTP request".to_string()))?;
        Ok(content)
    }
    pub async fn get_html_content(&self, url: &str) -> Result<String, Error> {
        let content = self
            .http_client
            .get(url)
            .send()
            .await
            .change_context_lazy(|| Error::Message("Failed to send HTTP request".to_string()))?
            .text()
            .await
            .change_context_lazy(|| {
                Error::Message("Failed to parse response body as text".to_string())
            })?;
        Ok(content)
    }

    pub async fn post_json<Body>(&self, url: &str, body: &Body) -> Result<reqwest::Response, Error>
    where
        Body: serde::Serialize,
    {
        let content = self
            .http_client
            .post(url)
            .json(body)
            .send()
            .await
            .change_context_lazy(|| Error::Message("Failed to send HTTP request".to_string()))?;
        Ok(content)
    }
}
