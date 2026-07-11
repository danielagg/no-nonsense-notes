use rusqlite::Connection;

const SCHEMA_VERSION_TABLE: &str = "\
    CREATE TABLE IF NOT EXISTS _schema_version (\
        version INTEGER PRIMARY KEY,\
        description TEXT NOT NULL,\
        applied_at TEXT NOT NULL DEFAULT (datetime('now'))\
    )";

struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "create notes, folders, tags, note_tags, settings tables",
    sql: r#"
        CREATE TABLE notes (
            id                  TEXT PRIMARY KEY,
            folder_id           TEXT REFERENCES folders(id),
            title               TEXT NOT NULL DEFAULT 'Untitled',
            content_plaintext   TEXT NOT NULL DEFAULT '',
            content_loro_blob   BLOB NOT NULL DEFAULT (x''),
            content_hash        BLOB NOT NULL DEFAULT (x''),
            created_at          TEXT NOT NULL,
            updated_at          TEXT NOT NULL,
            is_deleted          INTEGER NOT NULL DEFAULT 0,
            deleted_at          TEXT,
            sort_order          INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE folders (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL UNIQUE,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL
        );

        CREATE TABLE tags (
            id      TEXT PRIMARY KEY,
            name    TEXT NOT NULL UNIQUE,
            color   TEXT
        );

        CREATE TABLE note_tags (
            note_id TEXT NOT NULL REFERENCES notes(id),
            tag_id  TEXT NOT NULL REFERENCES tags(id),
            PRIMARY KEY (note_id, tag_id)
        );

        CREATE TABLE settings (
            key     TEXT PRIMARY KEY,
            value   TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE notes_fts USING fts5(
            title, content_plaintext
        );
    "#,
}];

pub fn run(conn: &Connection) -> Result<i64, crate::StorageError> {
    conn.execute_batch(SCHEMA_VERSION_TABLE)?;

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
        assert_eq!(version, 1);
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
}
