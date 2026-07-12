use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::Database;
use crate::error::ServerError;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub account_id: String,
}

#[derive(Deserialize)]
pub struct SigninRequest {
    pub email: String,
    pub password: String,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let body = format!("{}", self);
        let status = StatusCode::from(&self);
        (status, body).into_response()
    }
}

pub async fn signup(
    State(db): State<std::sync::Arc<Database>>,
    Json(req): Json<SignupRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), ServerError> {
    let password_hash = hash_password(&req.password)
        .map_err(ServerError::Internal)?;

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
        Json(AuthResponse {
            token,
            account_id,
        }),
    ))
}

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

    if !verify_password(&req.password, &stored_hash)
        .map_err(ServerError::Internal)?
    {
        return Err(ServerError::Unauthorized);
    }

    let token = generate_token();

    conn.execute(
        "INSERT INTO auth_tokens (token, account_id) VALUES (?1, ?2)",
        rusqlite::params![token, account_id],
    )
    .map_err(ServerError::Database)?;

    Ok(Json(AuthResponse {
        token,
        account_id,
    }))
}

pub fn verify_token(db: &Database, token: &str) -> Result<String, ServerError> {
    let conn = db.conn.lock().unwrap();
    let account_id: String = conn
        .query_row(
            "SELECT account_id FROM auth_tokens WHERE token = ?1",
            [token],
            |row| row.get(0),
        )
        .map_err(|_| ServerError::Unauthorized)?;
    Ok(account_id)
}

fn hash_password(password: &str) -> Result<String, String> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;

    let parsed = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    hex::encode(bytes)
}

mod hex {
    pub fn encode(bytes: Vec<u8>) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
