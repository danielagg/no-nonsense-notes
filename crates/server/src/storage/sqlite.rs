use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::ServerError;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, ServerError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        super::migrations::run(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn open_in_memory() -> Result<Self, ServerError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        super::migrations::run(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::migrations;
    use super::*;

    #[test]
    fn open_in_memory_creates_tables() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _schema_version",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, migrations::migration_count() as i64);
    }
}
