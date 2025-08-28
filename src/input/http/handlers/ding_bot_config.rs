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
        models::ding_bot::{
            CreateDingBotRequest, DingAccessToken, DingAccessTokenError, DingBotConfig,
            DingSecretToken, DingSecretTokenError,
        },
        ports::VulnService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateDingBotConfigHttpRequestBody {
    pub access_token: String,
    pub secret_token: String,
    pub status: bool,
}

impl CreateDingBotConfigHttpRequestBody {
    pub fn try_into_domain(
        self,
    ) -> Result<CreateDingBotRequest, ParseCreateDingBotConfigHttpRequestBodyError> {
        let access_token = DingAccessToken::try_new(self.access_token)?;
        let secret_token = DingSecretToken::try_new(self.secret_token)?;
        Ok(CreateDingBotRequest {
            access_token,
            secret_token,
            status: self.status,
        })
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseCreateDingBotConfigHttpRequestBodyError {
    #[error(transparent)]
    InvalidAccessToken(#[from] DingAccessTokenError),
    #[error(transparent)]
    InvalidSecretToken(#[from] DingSecretTokenError),
}

impl From<ParseCreateDingBotConfigHttpRequestBodyError> for ApiError {
    fn from(parse_error: ParseCreateDingBotConfigHttpRequestBodyError) -> Self {
        let message = match parse_error {
            ParseCreateDingBotConfigHttpRequestBodyError::InvalidAccessToken(e) => {
                format!("Access token is invalid: {}", e)
            }
            ParseCreateDingBotConfigHttpRequestBodyError::InvalidSecretToken(e) => {
                format!("Secret token is invalid: {}", e)
            }
        };
        ApiError::UnprocessableEntity(message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateDingBotConfigHttpResponseData {
    id: i64,
}

#[handler]
pub async fn create_or_update_ding_bot_config<S: VulnService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Json(body): Json<CreateDingBotConfigHttpRequestBody>,
) -> Result<ApiSuccess<CreateDingBotConfigHttpResponseData>, ApiError> {
    let req = body.try_into_domain()?;
    state
        .vuln_service
        .create_ding_bot_config(req)
        .await
        .map_err(ApiError::from)
        .map(|id| ApiSuccess::new(StatusCode::OK, CreateDingBotConfigHttpResponseData { id }))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetDingBotConfigHttpResponseData {
    pub id: i64,
    pub access_token: String,
    pub secret_token: String,
    pub status: bool,
}

impl From<DingBotConfig> for GetDingBotConfigHttpResponseData {
    fn from(config: DingBotConfig) -> Self {
        GetDingBotConfigHttpResponseData {
            id: config.id,
            access_token: config.access_token,
            secret_token: config.secret_token,
            status: config.status,
        }
    }
}

#[handler]
pub async fn get_ding_bot_config<S: VulnService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
) -> Result<ApiSuccess<Option<GetDingBotConfigHttpResponseData>>, ApiError> {
    state
        .vuln_service
        .get_ding_bot_config()
        .await
        .map_err(ApiError::from)
        .map(|data| ApiSuccess::new(StatusCode::OK, data.map(Into::into)))
}
