use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::sync_data_task::{
            CreateSyncDataTaskRequest, SyncDataTaskIntervalMinutes,
            SyncDataTaskIntervalMinutesError, SyncDataTaskName, SyncDataTaskNameError,
        },
        ports::VulnService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateSyncDataTaskHttpRequestBody {
    pub name: String,
    pub interval_minutes: u32,
    pub status: bool,
}

impl CreateSyncDataTaskHttpRequestBody {
    pub fn try_into_domain(
        self,
    ) -> Result<CreateSyncDataTaskRequest, ParseCreateSyncDataTaskRequestBodyError> {
        let name = SyncDataTaskName::try_new(self.name)?;
        let interval_minutes = SyncDataTaskIntervalMinutes::try_new(self.interval_minutes)?;
        Ok(CreateSyncDataTaskRequest {
            name,
            interval_minutes,
            status: self.status,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateSyncDataTaskHttpResponseData {
    pub id: i64,
}

#[derive(Debug, Clone, Error)]
pub enum ParseCreateSyncDataTaskRequestBodyError {
    #[error(transparent)]
    InvalidName(#[from] SyncDataTaskNameError),
    #[error(transparent)]
    InvalidIntervalMinutes(#[from] SyncDataTaskIntervalMinutesError),
}

impl From<ParseCreateSyncDataTaskRequestBodyError> for ApiError {
    fn from(parse_error: ParseCreateSyncDataTaskRequestBodyError) -> Self {
        let message = match parse_error {
            ParseCreateSyncDataTaskRequestBodyError::InvalidName(e) => {
                format!("Name is invalid: {}", e)
            }
            ParseCreateSyncDataTaskRequestBodyError::InvalidIntervalMinutes(e) => {
                format!("Interval minutes is invalid: {}", e)
            }
        };
        ApiError::UnprocessableEntity(message)
    }
}

#[handler]
pub async fn create_sync_data_task<S: VulnService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Json(body): Json<CreateSyncDataTaskHttpRequestBody>,
) -> Result<ApiSuccess<CreateSyncDataTaskHttpResponseData>, ApiError> {
    let req = body.try_into_domain()?;
    state
        .vuln_service
        .create_sync_data_task(req)
        .await
        .map_err(ApiError::from)
        .map(|id| {
            ApiSuccess::new(
                StatusCode::CREATED,
                CreateSyncDataTaskHttpResponseData { id },
            )
        })
}
