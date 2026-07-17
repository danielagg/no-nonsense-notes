use rusqlite::Connection;

use crate::StorageError;
use migration_build::Migration;

include!(concat!(env!("OUT_DIR"), "/migrations.rs"));

pub fn run(conn: &Connection) -> Result<i64, StorageError> {
    migration_build::run(conn, &MIGRATIONS).map_err(StorageError::from)
}

pub fn migration_count() -> usize {
    MIGRATION_COUNT
}

#[cfg(test)]
mod tests;
