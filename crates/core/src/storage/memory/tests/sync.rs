use super::super::*;

#[test]
fn apply_remote_update_creates_new_note() {
    let mut store = MemoryStore::new();

    let doc = LoroDoc::new();
    let text = doc.get_text("content");
    text.insert(0, "# Remote Note\n\nHello from another device")
        .unwrap();
    Note::set_title_in_doc(&doc, "Remote Note").unwrap();
    let blob = doc.export(ExportMode::Snapshot).unwrap();

    let remote_id = NoteId::now_v7();
    let note = store
        .apply_remote_update(remote_id, NoteType::Markdown, &blob)
        .unwrap();

    assert_eq!(note.id, remote_id);
    assert_eq!(note.note_type, NoteType::Markdown);
    assert_eq!(note.title, "Remote Note");
    assert!(note.content_plaintext.contains("Hello from another device"));

    let fetched = store.get(remote_id).unwrap();
    assert_eq!(fetched.id, remote_id);
}

#[test]
fn apply_remote_update_merges_into_existing() {
    let mut store = MemoryStore::new();
    let note = store.create(NoteType::Markdown, None).unwrap();
    store.update(note.id, "# Original\n\nHello", None).unwrap();

    let doc = LoroDoc::from_snapshot(&store.get(note.id).unwrap().content_loro_blob).unwrap();
    let text = doc.get_text("content");
    let pos = text.to_string().len();
    text.insert(pos, "\n\nAppended by remote").unwrap();
    let update_blob = doc.export(ExportMode::Snapshot).unwrap();

    let merged = store
        .apply_remote_update(note.id, NoteType::Markdown, &update_blob)
        .unwrap();

    assert!(merged.content_plaintext.contains("Hello"));
    assert!(merged.content_plaintext.contains("Appended by remote"));
}

#[test]
fn apply_remote_update_list_note() {
    let mut store = MemoryStore::new();

    let doc = LoroDoc::new();
    let list = doc.get_list("items");
    list.push("milk").unwrap();
    list.push("eggs").unwrap();
    Note::set_title_in_doc(&doc, "Groceries").unwrap();
    let blob = doc.export(ExportMode::Snapshot).unwrap();

    let remote_id = NoteId::now_v7();
    let note = store
        .apply_remote_update(remote_id, NoteType::List, &blob)
        .unwrap();

    assert_eq!(note.note_type, NoteType::List);
    assert_eq!(note.content_plaintext, "milk\neggs");
    assert_eq!(note.title, "Groceries");
}

#[test]
fn title_is_part_of_remote_update_metadata() {
    let mut source = MemoryStore::new();
    let note = source.create(NoteType::Markdown, None).unwrap();
    let renamed = source
        .update(note.id, "# Heading", Some("User-owned title"))
        .unwrap();

    let mut destination = MemoryStore::new();
    let received = destination
        .apply_remote_update(note.id, NoteType::Markdown, &renamed.content_loro_blob)
        .unwrap();
    assert_eq!(received.title, "User-owned title");

    let edited = source.update(note.id, "# Different heading", None).unwrap();
    let received = destination
        .apply_remote_update(note.id, NoteType::Markdown, &edited.content_loro_blob)
        .unwrap();
    assert_eq!(received.title, "User-owned title");
}
