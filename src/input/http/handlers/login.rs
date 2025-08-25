use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::{
            admin_user::{
                AdminUserPassword, AdminUserPasswordError, AdminUsername, AdminUsernameError,
            },
            auth::LoginRequest,
        },
        ports::VulnService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LoginHttpRequestBody {
    pub username: String,
    pub password: String,
}

impl LoginHttpRequestBody {
    pub fn try_into_domain(self) -> Result<LoginRequest, ParseLoginHttpRequestBodyError> {
        let username = AdminUsername::try_new(self.username)?;
        let password = AdminUserPassword::try_new(self.password)?;
        Ok(LoginRequest::new(username, password))
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseLoginHttpRequestBodyError {
    #[error(transparent)]
    Username(#[from] AdminUsernameError),
    #[error(transparent)]
    Password(#[from] AdminUserPasswordError),
}

impl From<ParseLoginHttpRequestBodyError> for ApiError {
    fn from(e: ParseLoginHttpRequestBodyError) -> Self {
        let message = match e {
            ParseLoginHttpRequestBodyError::Username(e) => {
                format!("Username is invalid: {}", e)
            }
            ParseLoginHttpRequestBodyError::Password(e) => {
                format!("Password is invalid: {}", e)
            }
        };
        Self::UnprocessableEntity(message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginHttpResponseData {
    pub token: String,
    pub user_id: i64,
    pub expires_in: u64,
}

#[handler]
pub async fn login<S: VulnService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Json(body): Json<LoginHttpRequestBody>,
) -> Result<ApiSuccess<LoginHttpResponseData>, ApiError> {
    let req = body.try_into_domain()?;
    let expires_in = state.config.auth.jwt.expiration;
    let admin_user = state
        .vuln_service
        .login(&req)
        .await
        .map_err(ApiError::from)?;

    state
        .jwt
        .generate_token(expires_in, admin_user.id, Map::new())
        .map_err(ApiError::from)
        .map(|token| {
            ApiSuccess::new(
                StatusCode::OK,
                LoginHttpResponseData {
                    token,
                    user_id: admin_user.id,
                    expires_in,
                },
            )
        })
}
