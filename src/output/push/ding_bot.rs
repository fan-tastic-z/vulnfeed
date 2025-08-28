use std::time::SystemTime;

use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};
use error_stack::Result;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::{
    errors::Error,
    output::push::MessageBot,
    utils::{http_client::HttpClient, util::calc_hmac_sha256},
};

const DING_API_URL: &str = "https://oapi.dingtalk.com/robot/send";
const MSG_TYPE: &str = "markdown";

#[derive(Debug, Clone)]
pub struct DingBot {
    pub access_token: String,
    pub secret_token: String,
    pub http_client: HttpClient,
}

#[async_trait]
impl MessageBot for DingBot {
    async fn push_markdown(&self, title: String, msg: String) -> Result<(), Error> {
        let msg = msg.replace("\n\n", "\n\n&nbsp;\n");
        let message = serde_json::json!({
            "msgtype": MSG_TYPE,
            "markdown": {
                "title": title,
                "text": msg
            },
        });

        let sign = self.generate_sign()?;
        let ding_response: DingResponse = self
            .http_client
            .http_client
            .post(DING_API_URL)
            .query(&sign)
            .json(&message)
            .send()
            .await
            .map_err(|e| Error::Message(format!("send ding message failed: {}", e)))?
            .json()
            .await
            .map_err(|e| Error::Message(format!("parse ding response failed: {}", e)))?;
        if ding_response.errcode != 0 {
            log::warn!(
                "ding push markdown message error, err msg is {}",
                ding_response.errmsg
            );
            return Err(Error::Message(
                "ding push markdown message response errorcode not eq 0".to_owned(),
            )
            .into());
        }
        Ok(())
    }
}

impl DingBot {
    pub fn try_new(access_token: String, secret_token: String) -> Result<Self, Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert("Accept-Charset", header::HeaderValue::from_static("utf8"));
        let http_client = HttpClient::try_new_with_headers(headers)?;
        Ok(Self {
            access_token,
            secret_token,
            http_client,
        })
    }

    pub fn generate_sign(&self) -> Result<Sign, Error> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| Error::Message(format!("get timestamp failed: {}", e)))?
            .as_millis();
        let timestamp_and_secret = &format!("{}\n{}", timestamp, self.secret_token);
        let hmac_sha256 = calc_hmac_sha256(
            self.secret_token.as_bytes(),
            timestamp_and_secret.as_bytes(),
        )?;
        let sign = BASE64_STANDARD.encode(hmac_sha256);
        Ok(Sign {
            access_token: self.access_token.clone(),
            timestamp,
            sign,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DingResponse {
    pub errmsg: String,
    pub errcode: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sign {
    pub access_token: String,
    pub timestamp: u128,
    pub sign: String,
}
