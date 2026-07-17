use super::super::*;

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
    assert_eq!(
        tables,
        vec![
            "folders",
            "note_tags",
            "notes",
            "pending_sync",
            "settings",
            "tags"
        ]
    );
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

    assert_eq!(rows.len(), migration_count());
    assert_eq!(rows[0].0, 1);
    assert_eq!(rows[1].0, 2);
}
