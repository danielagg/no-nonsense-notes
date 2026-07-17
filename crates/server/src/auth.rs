use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ServerError;

#[derive(Deserialize, ToSchema)]
pub struct SignupRequest {
    /// User email address
    pub email: String,
    /// User password
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    /// Authentication token
    pub token: String,
    /// Account UUID
    pub account_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SigninRequest {
    /// User email address
    pub email: String,
    /// User password
    pub password: String,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let body = format!("{}", self);
        let status = StatusCode::from(&self);
        (status, body).into_response()
    }
}

#[doc(hidden)]
pub mod handlers;
mod password;
mod token;

pub use handlers::{signin, signup};
pub use token::verify_token;
