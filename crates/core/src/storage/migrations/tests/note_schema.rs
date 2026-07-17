use super::super::*;

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
