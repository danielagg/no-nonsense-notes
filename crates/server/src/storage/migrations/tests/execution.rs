use super::super::*;

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
    assert_eq!(version, migration_count() as i64);
}

#[test]
fn migrations_are_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    run(&conn).unwrap();
    run(&conn).unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM _schema_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, migration_count() as i64);
}
