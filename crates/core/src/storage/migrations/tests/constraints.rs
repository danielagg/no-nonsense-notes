use super::super::*;

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
