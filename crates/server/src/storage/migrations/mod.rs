use rusqlite::Connection;

use crate::error::ServerError;
use migration_build::Migration;

include!(concat!(env!("OUT_DIR"), "/migrations.rs"));

pub fn run(conn: &Connection) -> Result<(), ServerError> {
    migration_build::run(conn, &MIGRATIONS).map_err(ServerError::Database)?;
    Ok(())
}

pub fn migration_count() -> usize {
    MIGRATION_COUNT
}

#[cfg(test)]
mod tests;
