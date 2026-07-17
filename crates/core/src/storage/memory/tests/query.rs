use super::super::*;

#[test]
fn soft_delete_hides_note() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();
    store.soft_delete(note.id).unwrap();

    let list = store.list(None).unwrap();
    assert!(list.is_empty());
}

#[test]
fn remote_delete_is_idempotent_for_existing_and_missing_notes() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();

    store.apply_remote_delete(note.id);
    store.apply_remote_delete(note.id);
    store.apply_remote_delete(NoteId::now_v7());

    assert!(store.list(None).unwrap().is_empty());
    assert!(store.get(note.id).unwrap().is_deleted);
}

#[test]
fn search_notes() {
    let mut store = MemoryStore::new();
    let n1 = store.create(NoteType::Markdown, None).unwrap();
    store
        .update(n1.id, "Groceries: milk and eggs", None)
        .unwrap();
    let n2 = store.create(NoteType::Markdown, None).unwrap();
    store.update(n2.id, "Meeting with Alice", None).unwrap();

    let results = store.search("Groceries").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, n1.id);
}

#[test]
fn get_missing_returns_not_found() {
    let store = MemoryStore::new();
    let id = NoteId::now_v7();
    let err = store.get(id).unwrap_err();
    assert!(matches!(err, StorageError::NotFound { .. }));
}

#[test]
fn list_filtered_by_folder() {
    let mut store = MemoryStore::new();
    let folder_id = NoteId::now_v7();

    store.create(NoteType::Markdown, None).unwrap();
    store.create(NoteType::List, Some(folder_id)).unwrap();
    store.create(NoteType::Markdown, Some(folder_id)).unwrap();

    let all = store.list(None).unwrap();
    assert_eq!(all.len(), 3);

    let filtered = store.list(Some(folder_id)).unwrap();
    assert_eq!(filtered.len(), 2);
}
