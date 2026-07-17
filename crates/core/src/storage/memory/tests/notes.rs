use super::super::*;

#[test]
fn create_markdown_and_get() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();
    assert_eq!(note.note_type, NoteType::Markdown);
    assert_eq!(note.title, "Untitled");
    assert!(!note.content_loro_blob.is_empty());

    let fetched = store.get(note.id).unwrap();
    assert_eq!(fetched.id, note.id);
    assert_eq!(fetched.title, "Untitled");
}

#[test]
fn create_list_and_get() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::List, None).unwrap();
    assert_eq!(note.note_type, NoteType::List);
    assert_eq!(note.title, "List");

    let fetched = store.get(note.id).unwrap();
    assert_eq!(fetched.note_type, NoteType::List);
}

#[test]
fn update_content_round_trip() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();
    let content = "# Hello\n\nThis is **bold** and *italic*.";
    let updated = store.update(note.id, content, None).unwrap();
    assert_eq!(updated.content_plaintext, content);
    assert_eq!(updated.title, "Untitled");

    let doc = LoroDoc::from_snapshot(&updated.content_loro_blob).unwrap();
    assert_eq!(doc.get_text("content").to_string(), content);
}

#[test]
fn update_rejects_list_type() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::List, None).unwrap();
    let err = store.update(note.id, "# Hello", None).unwrap_err();
    assert!(matches!(err, StorageError::WrongNoteType { .. }));
}

#[test]
fn update_with_title_override() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();
    let updated = store
        .update(note.id, "# Hello", Some("Custom Title"))
        .unwrap();
    assert_eq!(updated.title, "Custom Title");
}

#[test]
fn update_with_empty_title_uses_neutral_default() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();
    let updated = store.update(note.id, "# Hello", Some("  ")).unwrap();
    assert_eq!(updated.title, "Untitled");
}

#[test]
fn content_update_without_title_preserves_custom_title() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();
    store
        .update(note.id, "# First", Some("User Title"))
        .unwrap();
    let updated = store.update(note.id, "# World", None).unwrap();
    assert_eq!(updated.title, "User Title");
}
