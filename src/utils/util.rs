use chrono::DateTime;
use error_stack::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tera::{Context, Tera};

use crate::errors::Error;

pub fn timestamp_to_date(timestamp: i64) -> Result<String, Error> {
    let dt = DateTime::from_timestamp_millis(timestamp);
    if let Some(dt) = dt {
        return Ok(dt.format("%Y-%m-%d").to_string());
    }
    Err(Error::Message("convert timestamp to date error".to_string()).into())
}

pub fn calc_hmac_sha256(key: &[u8], message: &[u8]) -> Result<Vec<u8>, Error> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| Error::Message(format!("create hmac error: {:?}", e).to_string()))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

pub fn render_string(tera_template: &str, locals: &serde_json::Value) -> Result<String, Error> {
    let text = Tera::one_off(
        tera_template,
        &Context::from_serialize(locals)
            .map_err(|e| Error::Message(format!("render template error: {:?}", e).to_string()))?,
        false,
    )
    .map_err(|e| Error::Message(format!("render template error: {:?}", e).to_string()))?;
    Ok(text)
}
