use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use no_nonsense_notes_core::note::{Note, NoteId, NoteType};
use no_nonsense_notes_core::storage::memory::MemoryStore;
use no_nonsense_notes_core::sync::protocol;

const STORAGE_KEY: &str = "no-nonsense-notes-store";
const SYNC_CURSOR_KEY: &str = "no-nonsense-notes-sync-cursor";
const DEVICE_ID_KEY: &str = "no-nonsense-notes-device-id";

type JsResult<T> = Result<T, JsValue>;

#[derive(Serialize, Deserialize)]
struct StoreData {
    notes: Vec<Note>,
    next_sort_order: i64,
}

#[wasm_bindgen]
pub struct WasmStore {
    inner: MemoryStore,
    storage_namespace: String,
}

#[wasm_bindgen]
impl WasmStore {
    #[wasm_bindgen(constructor)]
    pub fn new(account_id: String) -> Self {
        remove_local_storage_item(STORAGE_KEY);
        remove_local_storage_item(SYNC_CURSOR_KEY);
        remove_local_storage_item(DEVICE_ID_KEY);

        let storage_namespace = account_id;
        let mut inner = MemoryStore::new();
        Self::load_from_storage(&mut inner, &storage_namespace);
        Self {
            inner,
            storage_namespace,
        }
    }

    #[wasm_bindgen(js_name = createNote)]
    pub fn create_note(&mut self, note_type: &str, folder_id: Option<String>) -> JsResult<JsValue> {
        let nt: NoteType = note_type
            .parse()
            .map_err(|e: String| JsValue::from_str(&e))?;
        let fid = folder_id
            .map(|s| s.parse::<NoteId>())
            .transpose()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let note = self
            .inner
            .create(nt, fid)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.save_to_storage();
        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = getNote)]
    pub fn get_note(&self, id: &str) -> JsResult<JsValue> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let note = self
            .inner
            .get(id)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = updateNote)]
    pub fn update_note(
        &mut self,
        id: &str,
        content: &str,
        title: Option<String>,
    ) -> JsResult<JsValue> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let note = self
            .inner
            .update(id, content, title.as_deref())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.save_to_storage();
        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = updateList)]
    pub fn update_list(
        &mut self,
        id: &str,
        items_json: &str,
        title: Option<String>,
    ) -> JsResult<JsValue> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let items: Vec<String> =
            serde_json::from_str(items_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let note = self
            .inner
            .list_replace_items(id, &items, title.as_deref())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.save_to_storage();
        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = listAddItem)]
    pub fn list_add_item(&mut self, id: &str, item: &str) -> JsResult<JsValue> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let note = self
            .inner
            .list_add_item(id, item)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.save_to_storage();
        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = listRemoveItem)]
    pub fn list_remove_item(&mut self, id: &str, item: &str) -> JsResult<JsValue> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let note = self
            .inner
            .list_remove_item(id, item)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.save_to_storage();
        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = softDelete)]
    pub fn soft_delete(&mut self, id: &str) -> JsResult<()> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner
            .soft_delete(id)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.save_to_storage();
        Ok(())
    }

    #[wasm_bindgen(js_name = listNotes)]
    pub fn list_notes(&self, folder_id: Option<String>) -> JsResult<JsValue> {
        let fid = folder_id
            .map(|s| s.parse::<NoteId>())
            .transpose()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let notes = self
            .inner
            .list(fid)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        notes_to_js(&notes)
    }

    #[wasm_bindgen(js_name = searchNotes)]
    pub fn search_notes(&self, query: &str) -> JsResult<JsValue> {
        let notes = self
            .inner
            .search(query)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        notes_to_js(&notes)
    }

    #[wasm_bindgen(js_name = applyRemoteUpdate)]
    pub fn apply_remote_update(
        &mut self,
        note_id: &str,
        note_type: &str,
        update_blob: &[u8],
    ) -> JsResult<JsValue> {
        let id = note_id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let nt: NoteType = note_type
            .parse()
            .map_err(|e: String| JsValue::from_str(&e))?;

        let note = self
            .inner
            .apply_remote_update(id, nt, update_blob)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.save_to_storage();
        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = getSyncCursor)]
    pub fn get_sync_cursor(&self) -> i64 {
        get_local_storage_item(&self.storage_key(SYNC_CURSOR_KEY))
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    }

    #[wasm_bindgen(js_name = setSyncCursor)]
    pub fn set_sync_cursor(&self, seq: i64) {
        set_local_storage_item(&self.storage_key(SYNC_CURSOR_KEY), &seq.to_string());
    }

    #[wasm_bindgen(js_name = getDeviceId)]
    pub fn get_device_id(&self) -> String {
        let key = self.storage_key(DEVICE_ID_KEY);
        if let Some(id) = get_local_storage_item(&key) {
            return id;
        }
        let id = NoteId::now_v7().to_string();
        set_local_storage_item(&key, &id);
        id
    }

    #[wasm_bindgen(js_name = exportNoteBlob)]
    pub fn export_note_blob(&self, note_id: &str) -> JsResult<Vec<u8>> {
        let id = note_id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let note = self
            .inner
            .get(id)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(note.content_loro_blob)
    }
}

impl WasmStore {
    fn storage_key(&self, base: &str) -> String {
        format!("{base}:{}", self.storage_namespace)
    }

    fn save_to_storage(&self) {
        let notes = self.inner.list(None).unwrap_or_default();
        let data = StoreData {
            notes,
            next_sort_order: self.inner.next_sort_order(),
        };
        if let Ok(json) = serde_json::to_string(&data) {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(&self.storage_key(STORAGE_KEY), &json);
                }
            }
        }
    }

    fn load_from_storage(inner: &mut MemoryStore, storage_namespace: &str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = window.local_storage() else {
            return;
        };
        let key = format!("{STORAGE_KEY}:{storage_namespace}");
        let Ok(Some(json)) = storage.get_item(&key) else {
            return;
        };
        let Ok(data) = serde_json::from_str::<StoreData>(&json) else {
            return;
        };
        for note in data.notes {
            inner.import_note(note);
        }
        inner.set_next_sort_order(data.next_sort_order);
    }
}

fn get_local_storage_item(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

fn set_local_storage_item(key: &str, value: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}

fn remove_local_storage_item(key: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item(key);
        }
    }
}

#[wasm_bindgen(js_name = encodePushFrame)]
pub fn encode_push_frame(
    doc_id: &str,
    device_id: &str,
    note_type: &str,
    loro_blob: &[u8],
) -> JsResult<Vec<u8>> {
    let doc_id = doc_id
        .parse::<NoteId>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let device_id = device_id
        .parse::<NoteId>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let nt: NoteType = note_type
        .parse()
        .map_err(|e: String| JsValue::from_str(&e))?;

    let payload = protocol::PushPayload {
        doc_id,
        device_id,
        note_type: nt,
        loro_blob,
    };
    Ok(protocol::encode_push_frame(&payload))
}

#[wasm_bindgen(js_name = decodePushResponse)]
pub fn decode_push_response(data: &[u8]) -> JsResult<i64> {
    protocol::decode_push_response(data).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen(js_name = encodePullRequest)]
pub fn encode_pull_request(last_seq: i64) -> String {
    protocol::encode_pull_request(last_seq)
}

#[wasm_bindgen(js_name = decodePullResponse)]
pub fn decode_pull_response(text: &str) -> JsResult<JsValue> {
    let response = protocol::decode_pull_response(text).map_err(|e| JsValue::from_str(&e))?;

    let obj = js_sys::Object::new();
    set_field(
        &obj,
        "currentSeq",
        &JsValue::from_f64(response.current_seq as f64),
    )?;

    let entries = js_sys::Array::new();
    for entry in &response.entries {
        let entry_obj = js_sys::Object::new();
        set_field(
            &entry_obj,
            "docId",
            &JsValue::from_str(&entry.doc_id.to_string()),
        )?;
        set_field(
            &entry_obj,
            "noteType",
            &JsValue::from_str(entry.note_type.as_str()),
        )?;
        set_field(
            &entry_obj,
            "loroBlob",
            &js_sys::Uint8Array::from(&entry.loro_blob[..]),
        )?;
        entries.push(&entry_obj);
    }
    set_field(&obj, "entries", &entries)?;

    Ok(obj.into())
}

fn set_field(obj: &js_sys::Object, key: &str, val: &JsValue) -> JsResult<()> {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), val)?;
    Ok(())
}

fn note_to_js(note: &Note) -> JsResult<JsValue> {
    let obj = js_sys::Object::new();

    set_field(&obj, "id", &JsValue::from_str(&note.id.to_string()))?;
    set_field(
        &obj,
        "folderId",
        &note
            .folder_id
            .map(|f| JsValue::from_str(&f.to_string()))
            .unwrap_or(JsValue::NULL),
    )?;
    set_field(
        &obj,
        "noteType",
        &JsValue::from_str(note.note_type.as_str()),
    )?;
    set_field(&obj, "title", &JsValue::from_str(&note.title))?;
    set_field(
        &obj,
        "contentPlaintext",
        &JsValue::from_str(&note.content_plaintext),
    )?;
    set_field(
        &obj,
        "contentLoroBlob",
        &js_sys::Uint8Array::from(&note.content_loro_blob[..]),
    )?;
    set_field(
        &obj,
        "contentHash",
        &js_sys::Uint8Array::from(&note.content_hash[..]),
    )?;
    set_field(
        &obj,
        "createdAt",
        &JsValue::from_str(&note.created_at.to_rfc3339()),
    )?;
    set_field(
        &obj,
        "updatedAt",
        &JsValue::from_str(&note.updated_at.to_rfc3339()),
    )?;
    set_field(&obj, "isDeleted", &JsValue::from_bool(note.is_deleted))?;
    set_field(
        &obj,
        "deletedAt",
        &note
            .deleted_at
            .map(|d| JsValue::from_str(&d.to_rfc3339()))
            .unwrap_or(JsValue::NULL),
    )?;
    set_field(
        &obj,
        "sortOrder",
        &JsValue::from_f64(note.sort_order as f64),
    )?;

    Ok(obj.into())
}

fn notes_to_js(notes: &[Note]) -> JsResult<JsValue> {
    let arr = js_sys::Array::new();
    for note in notes {
        arr.push(&note_to_js(note)?);
    }
    Ok(arr.into())
}

#[wasm_bindgen(start)]
pub fn init() {
    web_sys::console::log_1(&"no-nonsense-notes-wasm loaded".into());
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

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

        let updated = store.update_note(&id, "# Hello World", None).unwrap();
        let title = js_sys::Reflect::get(&updated, &JsValue::from_str("title"))
            .unwrap()
            .as_string()
            .unwrap();
        assert_eq!(title, "Hello World");

        let content = js_sys::Reflect::get(&updated, &JsValue::from_str("contentPlaintext"))
            .unwrap()
            .as_string()
            .unwrap();
        assert_eq!(content, "# Hello World");
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
        let updated = store.update_list(&id, items, None).unwrap();
        let content = js_sys::Reflect::get(&updated, &JsValue::from_str("contentPlaintext"))
            .unwrap()
            .as_string()
            .unwrap();
        assert_eq!(content, "milk\neggs\nbread");

        let title = js_sys::Reflect::get(&updated, &JsValue::from_str("title"))
            .unwrap()
            .as_string()
            .unwrap();
        assert_eq!(title, "milk");
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

    #[wasm_bindgen_test]
    fn apply_remote_update_creates_note() {
        let mut store = WasmStore::new(NoteId::now_v7().to_string());
        let src_note = store.create_note("markdown", None).unwrap();
        let src_id = js_sys::Reflect::get(&src_note, &JsValue::from_str("id"))
            .unwrap()
            .as_string()
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

    #[wasm_bindgen_test]
    fn encode_push_frame_produces_valid_frame() {
        let doc_id = NoteId::now_v7().to_string();
        let device_id = NoteId::now_v7().to_string();
        let blob = vec![0xDE, 0xAD, 0xBE, 0xEF];

        let frame = encode_push_frame(&doc_id, &device_id, "markdown", &blob).unwrap();
        assert!(frame.len() > 38);
        assert_eq!(frame[0], 1);
        assert_eq!(frame[1], 1);
    }

    #[wasm_bindgen_test]
    fn decode_push_response_round_trip() {
        let seq_bytes = 99i64.to_le_bytes();
        let seq = decode_push_response(&seq_bytes).unwrap();
        assert_eq!(seq, 99);
    }

    #[wasm_bindgen_test]
    fn encode_pull_request_format() {
        let req = encode_pull_request(123);
        assert_eq!(req, "pull:123");
    }

    #[wasm_bindgen_test]
    fn decode_pull_response_empty() {
        let response = decode_pull_response("seq:50\n").unwrap();
        let current_seq = js_sys::Reflect::get(&response, &JsValue::from_str("currentSeq"))
            .unwrap()
            .as_f64()
            .unwrap();
        assert_eq!(current_seq, 50.0);

        let entries = js_sys::Reflect::get(&response, &JsValue::from_str("entries")).unwrap();
        let arr: js_sys::Array = entries.into();
        assert_eq!(arr.length(), 0);
    }
}
