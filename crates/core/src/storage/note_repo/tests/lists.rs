use super::super::*;

#[test]
fn list_replace_items_with_title_override() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::List, None).unwrap();
    let items = vec!["milk".to_string(), "eggs".to_string()];
    let updated = repo
        .list_replace_items(note.id, &items, Some("Shopping"))
        .unwrap();
    assert_eq!(updated.title, "Shopping");
}

#[test]
fn list_add_and_remove() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::List, None).unwrap();
    let id = note.id;
    repo.list_replace_items(id, &[], Some("Shopping")).unwrap();

    repo.list_add_item(id, "milk").unwrap();
    repo.list_add_item(id, "eggs").unwrap();
    repo.list_add_item(id, "bread").unwrap();

    let note = repo.get(id).unwrap();
    assert_eq!(note.content_plaintext, "milk\neggs\nbread");
    assert_eq!(note.title, "Shopping");

    let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
    let list = doc.get_list("items");
    assert_eq!(list.len(), 3);

    let note = repo.list_remove_item(id, "eggs").unwrap();
    assert_eq!(note.content_plaintext, "milk\nbread");

    let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
    let list = doc.get_list("items");
    assert_eq!(list.len(), 2);
}

#[test]
fn list_replace_items_replaces_all() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::List, None).unwrap();
    repo.list_add_item(note.id, "milk").unwrap();
    repo.list_add_item(note.id, "eggs").unwrap();

    let new_items = vec!["coffee".to_string(), "sugar".to_string()];
    let note = repo.list_replace_items(note.id, &new_items, None).unwrap();
    assert_eq!(note.content_plaintext, "coffee\nsugar");
    assert_eq!(note.title, "List");

    let results = repo.search("coffee").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn list_replace_items_rejects_markdown() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::Markdown, None).unwrap();
    let err = repo
        .list_replace_items(note.id, &["x".to_string()], None)
        .unwrap_err();
    assert!(matches!(err, StorageError::WrongNoteType { .. }));
}

#[test]
fn list_replace_items_empty_clears_list() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::List, None).unwrap();
    repo.list_add_item(note.id, "milk").unwrap();
    repo.list_add_item(note.id, "eggs").unwrap();

    let note = repo.list_replace_items(note.id, &[], None).unwrap();
    assert_eq!(note.content_plaintext, "");
    assert_eq!(note.title, "List");

    let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
    let list = doc.get_list("items");
    assert_eq!(list.len(), 0);
}

#[test]
fn list_remove_missing_errors() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::List, None).unwrap();
    repo.list_add_item(note.id, "milk").unwrap();

    let err = repo.list_remove_item(note.id, "nope").unwrap_err();
    assert!(matches!(err, StorageError::NotFound { .. }));
}

#[test]
fn list_add_rejects_markdown() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::Markdown, None).unwrap();
    let err = repo.list_add_item(note.id, "milk").unwrap_err();
    assert!(matches!(err, StorageError::WrongNoteType { .. }));
}

#[test]
fn list_items_searchable() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let n1 = repo.create(NoteType::List, None).unwrap();
    repo.list_add_item(n1.id, "milk").unwrap();
    repo.list_add_item(n1.id, "eggs").unwrap();

    let n2 = repo.create(NoteType::Markdown, None).unwrap();
    repo.update(n2.id, "Meeting with Alice", None).unwrap();

    let results = repo.search("milk").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, n1.id);
    assert_eq!(results[0].note_type, NoteType::List);
}
