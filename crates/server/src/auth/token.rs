use crate::error::ServerError;
use crate::storage::Database;

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

pub(super) fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    hex::encode(bytes)
}

mod hex {
    pub fn encode(bytes: Vec<u8>) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
