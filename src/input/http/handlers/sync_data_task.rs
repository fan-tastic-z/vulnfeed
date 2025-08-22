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
            CreateSyncDataTaskRequest, SyncDataTask, SyncDataTaskIntervalMinutes,
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
            job_id: None,
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
pub async fn create_or_update_sync_data_task<S: VulnService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Json(body): Json<CreateSyncDataTaskHttpRequestBody>,
) -> Result<ApiSuccess<CreateSyncDataTaskHttpResponseData>, ApiError> {
    let req = body.try_into_domain()?;
    let id = state
        .vuln_service
        .create_sync_data_task(req)
        .await
        .map_err(ApiError::from)?;
    state.sched.update(id).await?;
    Ok(ApiSuccess::new(
        StatusCode::OK,
        CreateSyncDataTaskHttpResponseData { id },
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetSyncDataTaskHttpResponseData {
    pub id: i64,
    pub name: String,
    pub interval_minutes: i32,
    pub status: bool,
}

impl From<SyncDataTask> for GetSyncDataTaskHttpResponseData {
    fn from(sync_data_task: SyncDataTask) -> Self {
        GetSyncDataTaskHttpResponseData {
            id: sync_data_task.id,
            name: sync_data_task.name.to_string(),
            interval_minutes: sync_data_task.interval_minutes,
            status: sync_data_task.status,
        }
    }
}

#[handler]
pub async fn get_sync_data_task<S: VulnService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
) -> Result<ApiSuccess<Option<GetSyncDataTaskHttpResponseData>>, ApiError> {
    state
        .vuln_service
        .get_sync_data_task()
        .await
        .map_err(ApiError::from)
        .map(|data| ApiSuccess::new(StatusCode::OK, data.map(Into::into)))
}
