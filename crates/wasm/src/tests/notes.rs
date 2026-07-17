use super::super::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn create_markdown_note() {
    let mut store = WasmStore::new(NoteId::now_v7().to_string());
    let note = store.create_note("markdown", None).unwrap();
    let id = js_sys::Reflect::get(&note, &JsValue::from_str("id")).unwrap();
    assert!(id.is_string());
    let title = js_sys::Reflect::get(&note, &JsValue::from_str("title")).unwrap();
    assert_eq!(title.as_string().unwrap(), "Untitled");
}

#[wasm_bindgen_test]
fn create_and_update_markdown() {
    let mut store = WasmStore::new(NoteId::now_v7().to_string());
    let note = store.create_note("markdown", None).unwrap();
    let id = js_sys::Reflect::get(&note, &JsValue::from_str("id"))
        .unwrap()
        .as_string()
        .unwrap();

    let updated = store
        .update_note(&id, "# Hello World", Some("My title".to_string()))
        .unwrap();
    let title = js_sys::Reflect::get(&updated, &JsValue::from_str("title"))
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(title, "My title");

    let content = js_sys::Reflect::get(&updated, &JsValue::from_str("contentPlaintext"))
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(content, "# Hello World");

    let updated = store.update_note(&id, "# New heading", None).unwrap();
    let title = js_sys::Reflect::get(&updated, &JsValue::from_str("title"))
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(title, "My title");
}

#[wasm_bindgen_test]
fn create_list_and_replace_items() {
    let mut store = WasmStore::new(NoteId::now_v7().to_string());
    let note = store.create_note("list", None).unwrap();
    let id = js_sys::Reflect::get(&note, &JsValue::from_str("id"))
        .unwrap()
        .as_string()
        .unwrap();

    let items = r#"["milk","eggs","bread"]"#;
    let updated = store
        .update_list(&id, items, Some("Shopping".to_string()))
        .unwrap();
    let content = js_sys::Reflect::get(&updated, &JsValue::from_str("contentPlaintext"))
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(content, "milk\neggs\nbread");

    let title = js_sys::Reflect::get(&updated, &JsValue::from_str("title"))
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(title, "Shopping");

    let updated = store
        .update_list(&id, r#"["milk","eggs","bread","tea"]"#, None)
        .unwrap();
    let title = js_sys::Reflect::get(&updated, &JsValue::from_str("title"))
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(title, "Shopping");
}

#[wasm_bindgen_test]
fn soft_delete_removes_from_list() {
    let mut store = WasmStore::new(NoteId::now_v7().to_string());
    let note = store.create_note("markdown", None).unwrap();
    let id = js_sys::Reflect::get(&note, &JsValue::from_str("id"))
        .unwrap()
        .as_string()
        .unwrap();

    store.soft_delete(&id).unwrap();
    let list = store.list_notes(None).unwrap();
    let arr: js_sys::Array = list.into();
    assert_eq!(arr.length(), 0);
}

#[wasm_bindgen_test]
fn search_notes() {
    let mut store = WasmStore::new(NoteId::now_v7().to_string());
    let n1 = store.create_note("markdown", None).unwrap();
    let id1 = js_sys::Reflect::get(&n1, &JsValue::from_str("id"))
        .unwrap()
        .as_string()
        .unwrap();
    store
        .update_note(&id1, "Groceries: milk and eggs", None)
        .unwrap();

    let results = store.search_notes("Groceries").unwrap();
    let arr: js_sys::Array = results.into();
    assert_eq!(arr.length(), 1);
}
