use rusqlite::Connection;

use crate::error::ServerError;

struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/migrations.rs"));

pub fn run(conn: &Connection) -> Result<(), ServerError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_version (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
            [],
            |row| row.get(0),
        )
        .map_err(ServerError::Database)?;

    for migration in MIGRATIONS {
        if migration.version > current_version {
            tracing::info!(
                "applying migration {}: {}",
                migration.version,
                migration.description
            );
            conn.execute_batch(migration.sql)
                .map_err(|e| ServerError::Internal(format!("migration {} failed: {}", migration.version, e)))?;
            conn.execute(
                "INSERT INTO _schema_version (version, description) VALUES (?1, ?2)",
                rusqlite::params![migration.version, migration.description],
            )
            .map_err(ServerError::Database)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_applies() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        run(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM _schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn expected_tables_exist() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '_schema_version' AND name != 'sqlite_sequence'",
            )
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        let mut tables = tables;
        tables.sort();
        assert_eq!(tables, vec!["accounts", "auth_tokens", "updates"]);
    }

    #[test]
    fn schema_version_description_matches() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let description: String = conn
            .query_row(
                "SELECT description FROM _schema_version WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(description, "initial");
    }

    #[test]
    fn updates_table_has_autoincrement() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        conn.execute(
            "INSERT INTO updates (doc_id, device_id, blob) VALUES ('d1', 'dev1', X'01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO updates (doc_id, device_id, blob) VALUES ('d2', 'dev2', X'02')",
            [],
        )
        .unwrap();

        let seq1: i64 = conn
            .query_row(
                "SELECT global_seq FROM updates WHERE doc_id = 'd1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let seq2: i64 = conn
            .query_row(
                "SELECT global_seq FROM updates WHERE doc_id = 'd2'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
    }

    #[test]
    fn indexes_exist() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let indexes: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
            )
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        let mut indexes = indexes;
        indexes.sort();
        assert_eq!(
            indexes,
            vec!["idx_updates_device_id", "idx_updates_doc_id"]
        );
    }

    #[test]
    fn foreign_key_constraint_enforced() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn).unwrap();

        let result = conn.execute(
            "INSERT INTO auth_tokens (token, account_id) VALUES ('tok', 'nonexistent')",
            [],
        );
        assert!(result.is_err());
    }

    #[test]
    fn unique_email_constraint() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        conn.execute(
            "INSERT INTO accounts (id, email, password_hash) VALUES ('a1', 'test@example.com', 'hash')",
            [],
        )
        .unwrap();

        let result = conn.execute(
            "INSERT INTO accounts (id, email, password_hash) VALUES ('a2', 'test@example.com', 'hash')",
            [],
        );
        assert!(result.is_err());
    }
}
