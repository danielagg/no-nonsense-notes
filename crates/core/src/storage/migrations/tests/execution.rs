use super::super::*;

#[test]
fn migrations_apply_in_order() {
    let conn = Connection::open_in_memory().unwrap();
    let version = run(&conn).unwrap();
    assert_eq!(version, migration_count() as i64);
}

#[test]
fn migrations_are_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    let v1 = run(&conn).unwrap();
    let v2 = run(&conn).unwrap();
    assert_eq!(v1, v2);
}
