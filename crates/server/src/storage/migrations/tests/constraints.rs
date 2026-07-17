use super::super::*;

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
