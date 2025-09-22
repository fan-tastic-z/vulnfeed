use chrono::{DateTime, Utc};
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Query},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::{
            page_utils::{PageFilter, PageNo, PageNoError, PageSize, PageSizeError},
            security_notice::{ListSecNoticeRequest, SearchParams, SecuritNotice},
        },
        ports::VulnService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListSecNoticeRequestBody {
    pub page_no: i32,
    pub page_size: i32,
    pub title: Option<String>,
    pub pushed: Option<bool>,
    pub source_name: Option<String>,
}

impl ListSecNoticeRequestBody {
    pub fn try_into_domain(
        self,
    ) -> Result<ListSecNoticeRequest, ParseListSecNoticeRequestBodyError> {
        let page_no = PageNo::try_new(self.page_no)?;
        let page_size = PageSize::try_new(self.page_size)?;
        let page_filter = PageFilter::new(page_no, page_size);
        let search_params = SearchParams::new()
            .with_title(self.title)
            .with_pushed(self.pushed)
            .with_source_name(self.source_name);
        Ok(ListSecNoticeRequest::new(page_filter, search_params))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct SecNoticeData {
    pub id: i64,
    pub key: String,
    pub title: String,
    pub product_name: String,
    pub risk_level: String,
    pub source: String,
    pub source_name: String,
    pub is_zero_day: bool,
    pub publish_time: String,
    pub detail_link: String,
    pub pushed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SecNoticeData {
    pub fn new(notice: SecuritNotice) -> Self {
        Self {
            id: notice.id,
            key: notice.key,
            title: notice.title,
            product_name: notice.product_name,
            risk_level: notice.risk_level,
            source: notice.source,
            source_name: notice.source_name,
            is_zero_day: notice.is_zero_day,
            publish_time: notice.publish_time,
            detail_link: notice.detail_link,
            pushed: notice.pushed,
            created_at: notice.created_at,
            updated_at: notice.updated_at,
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseListSecNoticeRequestBodyError {
    #[error(transparent)]
    InvalidPageNo(#[from] PageNoError),
    #[error(transparent)]
    InvalidPageSize(#[from] PageSizeError),
}

impl From<ParseListSecNoticeRequestBodyError> for ApiError {
    fn from(e: ParseListSecNoticeRequestBodyError) -> Self {
        let msg = match e {
            ParseListSecNoticeRequestBodyError::InvalidPageNo(e) => {
                format!("Invalid page number: {}", e)
            }
            ParseListSecNoticeRequestBodyError::InvalidPageSize(e) => {
                format!("Invalid page size: {}", e)
            }
        };
        ApiError::UnprocessableEntity(msg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListSecNoticeHttpResponseData {
    pub data: Vec<SecNoticeData>,
    pub total_count: i64,
}

#[handler]
pub async fn list_sec_notice<S: VulnService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Query(body): Query<ListSecNoticeRequestBody>,
) -> Result<ApiSuccess<ListSecNoticeHttpResponseData>, ApiError> {
    let req = body.try_into_domain()?;
    state
        .vuln_service
        .list_sec_notice(req)
        .await
        .map_err(ApiError::from)
        .map(|data| {
            let response_data = ListSecNoticeHttpResponseData {
                data: data.data.into_iter().map(SecNoticeData::new).collect(),
                total_count: data.total,
            };
            ApiSuccess::new(StatusCode::OK, response_data)
        })
}
