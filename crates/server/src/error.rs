use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<&ServerError> for StatusCode {
    fn from(err: &ServerError) -> Self {
        match err {
            ServerError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::NotFound => StatusCode::NOT_FOUND,
            ServerError::Unauthorized => StatusCode::UNAUTHORIZED,
            ServerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServerError::Conflict(_) => StatusCode::CONFLICT,
            ServerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
