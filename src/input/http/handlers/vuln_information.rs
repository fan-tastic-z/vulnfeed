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
            vuln_information::{
                ListVulnInformationRequest, ListVulnInformationResponseData, VulnInformation,
            },
        },
        ports::VulnService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListVulnInformationRequestBody {
    pub page_no: i32,
    pub page_size: i32,
}

impl ListVulnInformationRequestBody {
    pub fn try_into_domain(
        self,
    ) -> Result<ListVulnInformationRequest, ParseListVulnInformationRequestBodyError> {
        let page_no = PageNo::try_new(self.page_no)?;
        let page_size = PageSize::try_new(self.page_size)?;
        let page_filter = PageFilter::new(page_no, page_size);
        Ok(ListVulnInformationRequest::new(page_filter))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct VulnInformationData {
    pub id: i64,
    pub key: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub cve: String,
    pub disclosure: String,
    pub solutions: String,
    pub reference_links: Vec<String>,
    pub tags: Vec<String>,
    pub github_search: Vec<String>,
    pub source: String,
    pub reasons: Vec<String>,
    pub pushed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VulnInformationData {
    pub fn new(vuln: VulnInformation) -> Self {
        Self {
            id: vuln.id,
            key: vuln.key,
            title: vuln.title,
            description: vuln.description,
            severity: vuln.severity,
            cve: vuln.cve,
            disclosure: vuln.disclosure,
            solutions: vuln.solutions,
            reference_links: vuln.reference_links,
            tags: vuln.tags,
            github_search: vuln.github_search,
            source: vuln.source,
            reasons: vuln.reasons,
            pushed: vuln.pushed,
            created_at: vuln.created_at,
            updated_at: vuln.updated_at,
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseListVulnInformationRequestBodyError {
    #[error(transparent)]
    InvalidPageNo(#[from] PageNoError),
    #[error(transparent)]
    InvalidPageSize(#[from] PageSizeError),
}

impl From<ParseListVulnInformationRequestBodyError> for ApiError {
    fn from(e: ParseListVulnInformationRequestBodyError) -> Self {
        let msg = match e {
            ParseListVulnInformationRequestBodyError::InvalidPageNo(e) => {
                format!("Invalid page number: {}", e)
            }
            ParseListVulnInformationRequestBodyError::InvalidPageSize(e) => {
                format!("Invalid page size: {}", e)
            }
        };
        ApiError::UnprocessableEntity(msg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListVulnInformationHttpResponseData {
    pub data: Vec<VulnInformationData>,
    pub total_count: i64,
}

impl From<ListVulnInformationResponseData> for ListVulnInformationHttpResponseData {
    fn from(vulns: ListVulnInformationResponseData) -> Self {
        Self {
            data: vulns
                .data
                .into_iter()
                .map(VulnInformationData::new)
                .collect(),
            total_count: vulns.total,
        }
    }
}

#[handler]
pub async fn list_vulnfusion_information<S: VulnService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Query(body): Query<ListVulnInformationRequestBody>,
) -> Result<ApiSuccess<ListVulnInformationHttpResponseData>, ApiError> {
    let req = body.try_into_domain()?;
    state
        .vuln_service
        .list_vulnfusion_information(req)
        .await
        .map_err(ApiError::from)
        .map(|data| ApiSuccess::new(StatusCode::OK, data.into()))
}
