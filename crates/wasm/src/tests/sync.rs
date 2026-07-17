use super::super::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn apply_remote_update_creates_note() {
    let mut store = WasmStore::new(NoteId::now_v7().to_string());
    let src_note = store.create_note("markdown", None).unwrap();
    let src_id = js_sys::Reflect::get(&src_note, &JsValue::from_str("id"))
        .unwrap()
        .as_string()
        .unwrap();
    store
        .update_note(&src_id, "Remote content", Some("Synced title".to_string()))
        .unwrap();
    let blob = store.export_note_blob(&src_id).unwrap();

    let remote_id = NoteId::now_v7().to_string();
    let note = store
        .apply_remote_update(&remote_id, "markdown", &blob)
        .unwrap();
    let id = js_sys::Reflect::get(&note, &JsValue::from_str("id"))
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(id, remote_id);
    let title = js_sys::Reflect::get(&note, &JsValue::from_str("title"))
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(title, "Synced title");
}

#[wasm_bindgen_test]
fn apply_remote_delete_hides_note_and_ignores_missing_note() {
    let mut store = WasmStore::new(NoteId::now_v7().to_string());
    let note = store.create_note("markdown", None).unwrap();
    let id = js_sys::Reflect::get(&note, &JsValue::from_str("id"))
        .unwrap()
        .as_string()
        .unwrap();

    store.apply_remote_delete(&id).unwrap();
    store
        .apply_remote_delete(&NoteId::now_v7().to_string())
        .unwrap();

    let notes: js_sys::Array = store.list_notes(None).unwrap().into();
    assert_eq!(notes.length(), 0);
}

#[wasm_bindgen_test]
fn sync_cursor_round_trip() {
    let store = WasmStore::new(NoteId::now_v7().to_string());
    store.set_sync_cursor(42);
    assert_eq!(store.get_sync_cursor(), 42);
}

#[wasm_bindgen_test]
fn device_id_stable() {
    let store = WasmStore::new(NoteId::now_v7().to_string());
    let id1 = store.get_device_id();
    let id2 = store.get_device_id();
    assert_eq!(id1, id2);
    assert!(!id1.is_empty());
}

#[wasm_bindgen_test]
fn sync_state_is_scoped_by_account() {
    let account_a = NoteId::now_v7().to_string();
    let account_b = NoteId::now_v7().to_string();
    let store_a = WasmStore::new(account_a);
    store_a.set_sync_cursor(42);
    let device_a = store_a.get_device_id();

    let store_b = WasmStore::new(account_b);
    assert_eq!(store_b.get_sync_cursor(), 0);
    assert_ne!(store_b.get_device_id(), device_a);
}
