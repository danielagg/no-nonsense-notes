use rusqlite::Connection;

struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/migrations.rs"));

pub fn run(conn: &Connection) -> Result<i64, crate::StorageError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_version (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for m in MIGRATIONS {
        if m.version <= current {
            continue;
        }
        conn.execute_batch(m.sql)?;
        conn.execute(
            "INSERT INTO _schema_version (version, description) VALUES (?1, ?2)",
            rusqlite::params![m.version, m.description],
        )?;
    }

    let final_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(final_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_in_order() {
        let conn = Connection::open_in_memory().unwrap();
        let version = run(&conn).unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        let v1 = run(&conn).unwrap();
        let v2 = run(&conn).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn expected_tables_exist() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '_schema_version' AND name NOT LIKE 'notes_fts%'",
            )
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        let mut tables = tables;
        tables.sort();
        assert_eq!(tables, vec!["folders", "note_tags", "notes", "settings", "tags"]);
    }

    #[test]
    fn schema_version_has_row_per_migration() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let rows: Vec<(i64, String)> = conn
            .prepare("SELECT version, description FROM _schema_version ORDER BY version")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, 1);
        assert_eq!(rows[1].0, 2);
    }

    #[test]
    fn migration_002_adds_note_type_column() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(notes)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert!(columns.contains(&"note_type".to_string()));
    }

    #[test]
    fn note_type_defaults_to_markdown() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        conn.execute(
            "INSERT INTO notes (id, title, content_plaintext, created_at, updated_at) VALUES ('n1', 't', '', 'now', 'now')",
            [],
        )
        .unwrap();

        let note_type: String = conn
            .query_row("SELECT note_type FROM notes WHERE id = 'n1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(note_type, "markdown");
    }

    #[test]
    fn notes_table_columns_match_expected() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(notes)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        let expected = vec![
            "id",
            "folder_id",
            "title",
            "content_plaintext",
            "content_loro_blob",
            "content_hash",
            "created_at",
            "updated_at",
            "is_deleted",
            "deleted_at",
            "sort_order",
            "note_type",
        ];
        assert_eq!(columns, expected);
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn).unwrap();

        let result = conn.execute(
            "INSERT INTO note_tags (note_id, tag_id) VALUES ('nonexistent', 'nonexistent')",
            [],
        );
        assert!(result.is_err());
    }
}
