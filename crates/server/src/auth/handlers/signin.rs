use axum::{Json, extract::State};

use crate::auth::{AuthResponse, SigninRequest};
use crate::error::ServerError;
use crate::storage::Database;

use super::super::password::verify_password;
use super::super::token::generate_token;

#[utoipa::path(
    post,
    path = "/auth/signin",
    request_body = SigninRequest,
    responses(
        (status = 200, description = "Signed in", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal error")
    )
)]
pub async fn signin(
    State(db): State<std::sync::Arc<Database>>,
    Json(req): Json<SigninRequest>,
) -> Result<Json<AuthResponse>, ServerError> {
    let conn = db.conn.lock().unwrap();

    let row: Result<(String, String), _> = conn.query_row(
        "SELECT id, password_hash FROM accounts WHERE email = ?1",
        [&req.email],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let (account_id, stored_hash) = match row {
        Ok(r) => r,
        Err(_) => return Err(ServerError::Unauthorized),
    };

    if !verify_password(&req.password, &stored_hash).map_err(ServerError::Internal)? {
        return Err(ServerError::Unauthorized);
    }

    let token = generate_token();

    conn.execute(
        "INSERT INTO auth_tokens (token, account_id) VALUES (?1, ?2)",
        rusqlite::params![token, account_id],
    )
    .map_err(ServerError::Database)?;

    Ok(Json(AuthResponse { token, account_id }))
}
