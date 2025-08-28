use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
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

pub fn get_last_year_data() -> String {
    let current_date = Local::now();
    let last_year = current_date - Duration::days(365);
    last_year.format("%Y-%m-%d").to_string()
}

pub fn check_over_two_week(date: &str) -> Result<bool, Error> {
    let target_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| Error::Message(format!("parse date error: {:?}", e)))?;
    let now = Utc::now().naive_utc().date();
    let two_weeks_ago = now - Duration::weeks(2);
    if target_date >= two_weeks_ago && target_date <= now {
        return Ok(false);
    }
    Ok(true)
}
