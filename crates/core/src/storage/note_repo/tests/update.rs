use super::super::*;

#[test]
fn update_content_round_trip() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::Markdown, None).unwrap();
    let content = "# Hello\n\nThis is **bold** and *italic*.";
    let updated = repo.update(note.id, content, None).unwrap();
    assert_eq!(updated.content_plaintext, content);
    assert_eq!(updated.title, "Untitled");

    let doc = LoroDoc::from_snapshot(&updated.content_loro_blob).unwrap();
    assert_eq!(doc.get_text("content").to_string(), content);
}

#[test]
fn update_rejects_list_type() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::List, None).unwrap();
    let err = repo.update(note.id, "# Hello", None).unwrap_err();
    assert!(matches!(err, StorageError::WrongNoteType { .. }));
}

#[test]
fn update_with_title_override() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::Markdown, None).unwrap();
    let updated = repo
        .update(note.id, "# Hello", Some("Custom Title"))
        .unwrap();
    assert_eq!(updated.title, "Custom Title");

    let updated = repo.update(note.id, "# Different heading", None).unwrap();
    assert_eq!(updated.title, "Custom Title");
    let doc = LoroDoc::from_snapshot(&updated.content_loro_blob).unwrap();
    assert_eq!(Note::title_from_doc(&doc).as_deref(), Some("Custom Title"));
}

#[test]
fn update_with_empty_title_uses_neutral_default() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::storage::migrations::run(&conn).unwrap();
    let repo = NoteRepository::new(&conn);

    let note = repo.create(NoteType::Markdown, None).unwrap();
    let updated = repo.update(note.id, "# Hello", Some("  ")).unwrap();
    assert_eq!(updated.title, "Untitled");
}
