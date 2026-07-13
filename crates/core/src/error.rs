use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[cfg(feature = "sqlite")]
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("migration {version} failed: {detail}")]
    Migration { version: i64, detail: String },

    #[error("loro error: {0}")]
    Loro(String),

    #[error("note {id} not found")]
    NotFound { id: String },

    #[error("wrong note type for operation: expected {expected}, got {actual}")]
    WrongNoteType { expected: String, actual: String },

    #[error("parse error: {0}")]
    Parse(String),

    #[error("protocol error: {0}")]
    Protocol(String),
}