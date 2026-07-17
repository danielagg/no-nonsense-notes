use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::auth::{AuthResponse, SignupRequest};
use crate::error::ServerError;
use crate::storage::Database;

use super::super::password::hash_password;
use super::super::token::generate_token;

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "Account created", body = AuthResponse),
        (status = 409, description = "Email already registered"),
        (status = 500, description = "Internal error")
    )
)]
pub async fn signup(
    State(db): State<std::sync::Arc<Database>>,
    Json(req): Json<SignupRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), ServerError> {
    let password_hash = hash_password(&req.password).map_err(ServerError::Internal)?;

    let account_id = Uuid::new_v4().to_string();
    let token = generate_token();

    let conn = db.conn.lock().unwrap();

    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM accounts WHERE email = ?1",
            [&req.email],
            |row| row.get(0),
        )
        .map_err(ServerError::Database)?;

    if exists {
        return Err(ServerError::Conflict("email already registered".into()));
    }

    conn.execute(
        "INSERT INTO accounts (id, email, password_hash) VALUES (?1, ?2, ?3)",
        rusqlite::params![account_id, req.email, password_hash],
    )
    .map_err(ServerError::Database)?;

    conn.execute(
        "INSERT INTO auth_tokens (token, account_id) VALUES (?1, ?2)",
        rusqlite::params![token, account_id],
    )
    .map_err(ServerError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse { token, account_id }),
    ))
}
