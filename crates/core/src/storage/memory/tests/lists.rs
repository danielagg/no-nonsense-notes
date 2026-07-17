use super::super::*;

#[test]
fn list_replace_items_with_title_override() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::List, None).unwrap();
    let items = vec!["milk".to_string(), "eggs".to_string()];
    let updated = store
        .list_replace_items(note.id, &items, Some("Shopping List"))
        .unwrap();
    assert_eq!(updated.title, "Shopping List");
}

#[test]
fn list_add_and_remove() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::List, None).unwrap();
    let id = note.id;
    store.list_replace_items(id, &[], Some("Shopping")).unwrap();

    store.list_add_item(id, "milk").unwrap();
    store.list_add_item(id, "eggs").unwrap();
    store.list_add_item(id, "bread").unwrap();

    let note = store.get(id).unwrap();
    assert_eq!(note.content_plaintext, "milk\neggs\nbread");
    assert_eq!(note.title, "Shopping");

    let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
    let list = doc.get_list("items");
    assert_eq!(list.len(), 3);

    let note = store.list_remove_item(id, "eggs").unwrap();
    assert_eq!(note.content_plaintext, "milk\nbread");
}

#[test]
fn list_replace_items_replaces_all() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::List, None).unwrap();
    let id = note.id;

    store.list_add_item(id, "milk").unwrap();
    store.list_add_item(id, "eggs").unwrap();

    let new_items = vec![
        "coffee".to_string(),
        "sugar".to_string(),
        "flour".to_string(),
    ];
    let note = store.list_replace_items(id, &new_items, None).unwrap();
    assert_eq!(note.content_plaintext, "coffee\nsugar\nflour");
    assert_eq!(note.title, "List");

    let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
    let list = doc.get_list("items");
    assert_eq!(list.len(), 3);
}

#[test]
fn list_replace_items_rejects_markdown() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();
    let err = store
        .list_replace_items(note.id, &["x".to_string()], None)
        .unwrap_err();
    assert!(matches!(err, StorageError::WrongNoteType { .. }));
}

#[test]
fn list_replace_items_empty_clears_list() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::List, None).unwrap();
    store.list_add_item(note.id, "milk").unwrap();
    store.list_add_item(note.id, "eggs").unwrap();

    let note = store.list_replace_items(note.id, &[], None).unwrap();
    assert_eq!(note.content_plaintext, "");
    assert_eq!(note.title, "List");

    let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
    let list = doc.get_list("items");
    assert_eq!(list.len(), 0);
}

#[test]
fn list_remove_missing_errors() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::List, None).unwrap();
    store.list_add_item(note.id, "milk").unwrap();

    let err = store.list_remove_item(note.id, "nope").unwrap_err();
    assert!(matches!(err, StorageError::NotFound { .. }));
}
