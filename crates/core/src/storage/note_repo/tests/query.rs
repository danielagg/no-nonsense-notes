use super::super::*;

#[test]
fn soft_delete_hides_note() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::Markdown, None).unwrap();
    repo.soft_delete(note.id).unwrap();

    let list = repo.list(None).unwrap();
    assert!(list.is_empty());
}

#[test]
fn list_notes() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    repo.create(NoteType::Markdown, None).unwrap();
    repo.create(NoteType::List, None).unwrap();
    repo.create(NoteType::Markdown, None).unwrap();

    let list = repo.list(None).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn list_filtered_by_folder() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let folder_id = NoteId::now_v7();
    conn.execute(
        "INSERT INTO folders (id, name, sort_order, created_at) VALUES (?1, ?2, 0, ?3)",
        params![
            folder_id.to_string(),
            "Work",
            chrono::Utc::now().to_rfc3339()
        ],
    )
    .unwrap();

    repo.create(NoteType::Markdown, None).unwrap();
    repo.create(NoteType::List, Some(folder_id)).unwrap();
    repo.create(NoteType::Markdown, Some(folder_id)).unwrap();

    let all = repo.list(None).unwrap();
    assert_eq!(all.len(), 3);

    let filtered = repo.list(Some(folder_id)).unwrap();
    assert_eq!(filtered.len(), 2);
}

#[test]
fn search_notes() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let n1 = repo.create(NoteType::Markdown, None).unwrap();
    repo.update(n1.id, "Groceries: milk and eggs", None)
        .unwrap();
    let n2 = repo.create(NoteType::Markdown, None).unwrap();
    repo.update(n2.id, "Meeting with Alice", None).unwrap();

    let results = repo.search("Groceries").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, n1.id);
}

#[test]
fn search_ignores_deleted() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::Markdown, None).unwrap();
    repo.update(note.id, "Important stuff", None).unwrap();
    repo.soft_delete(note.id).unwrap();

    let results = repo.search("Important").unwrap();
    assert!(results.is_empty());
}

#[test]
fn get_missing_returns_not_found() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let id = NoteId::now_v7();
    let err = repo.get(id).unwrap_err();
    assert!(matches!(err, StorageError::NotFound { .. }));
}
