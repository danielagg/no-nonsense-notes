use super::super::*;

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

    let description: String = conn
        .query_row(
            "SELECT description FROM _schema_version WHERE version = 2",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(description, "scope updates by account");
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
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    let mut indexes = indexes;
    indexes.sort();
    assert_eq!(
        indexes,
        vec![
            "idx_updates_account_id",
            "idx_updates_device_id",
            "idx_updates_doc_id"
        ]
    );
}
