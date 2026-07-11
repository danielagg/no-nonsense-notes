use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("migration {version} failed: {detail}")]
    Migration { version: i64, detail: String },

    #[error("loro error: {0}")]
    Loro(String),
}
