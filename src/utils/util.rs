use chrono::DateTime;
use error_stack::Result;

use crate::errors::Error;

pub fn timestamp_to_date(timestamp: i64) -> Result<String, Error> {
    let dt = DateTime::from_timestamp_millis(timestamp);
    if let Some(dt) = dt {
        return Ok(dt.format("%Y-%m-%d").to_string());
    }
    Err(Error::Message("convert timestamp to date error".to_string()).into())
}
