use super::super::*;
use crate::storage::sqlite::Database;

#[test]
fn create_markdown_and_get() {
    let db = Database::open_in_memory().unwrap();
    let repo = NoteRepository::new(db.connection());

    let note = repo.create(NoteType::Markdown, None).unwrap();
    assert_eq!(note.note_type, NoteType::Markdown);
    assert_eq!(note.title, "Untitled");
    assert!(!note.content_loro_blob.is_empty());

    let fetched = repo.get(note.id).unwrap();
    assert_eq!(fetched.id, note.id);
    assert_eq!(fetched.title, "Untitled");
    assert_eq!(fetched.note_type, NoteType::Markdown);
}

#[test]
fn create_list_and_get() {
    let db = Database::open_in_memory().unwrap();
    let repo = NoteRepository::new(db.connection());

    let note = repo.create(NoteType::List, None).unwrap();
    assert_eq!(note.note_type, NoteType::List);
    assert_eq!(note.title, "List");
    assert!(note.content_plaintext.is_empty());

    let fetched = repo.get(note.id).unwrap();
    assert_eq!(fetched.note_type, NoteType::List);
}

#[test]
fn create_with_folder() {
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

    let note = repo.create(NoteType::Markdown, Some(folder_id)).unwrap();
    assert_eq!(note.folder_id, Some(folder_id));
}

#[test]
fn content_does_not_derive_title() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::Markdown, None).unwrap();
    let updated = repo
        .update(note.id, "# Meeting Notes\n\nLorum ipsum.", None)
        .unwrap();
    assert_eq!(updated.title, "Untitled");
}
