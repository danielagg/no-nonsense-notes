use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use no_nonsense_notes_core::note::{Note, NoteId, NoteType};
use no_nonsense_notes_core::storage::memory::MemoryStore;

const STORAGE_KEY: &str = "no-nonsense-notes-store";

type JsResult<T> = Result<T, JsValue>;

#[derive(Serialize, Deserialize)]
struct StoreData {
    notes: Vec<Note>,
    next_sort_order: i64,
}

#[wasm_bindgen]
pub struct WasmStore {
    inner: MemoryStore,
}

#[wasm_bindgen]
impl WasmStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut inner = MemoryStore::new();
        Self::load_from_storage(&mut inner);
        Self { inner }
    }

    #[wasm_bindgen(js_name = createNote)]
    pub fn create_note(
        &mut self,
        note_type: &str,
        folder_id: Option<String>,
    ) -> JsResult<JsValue> {
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
    pub fn update_note(&mut self, id: &str, content: &str) -> JsResult<JsValue> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let note = self
            .inner
            .update(id, content)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.save_to_storage();
        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = updateList)]
    pub fn update_list(&mut self, id: &str, items_json: &str) -> JsResult<JsValue> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let items: Vec<String> = serde_json::from_str(items_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Remove all existing items, then add new ones
        let existing = self.inner.get(id)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let existing_items: Vec<String> = existing.content_plaintext
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        for item in &existing_items {
            self.inner.list_remove_item(id, item)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }
        for item in &items {
            self.inner.list_add_item(id, item)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }

        self.save_to_storage();
        let note = self.inner.get(id)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
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
}

impl WasmStore {
    fn save_to_storage(&self) {
        let notes = self.inner.list(None).unwrap_or_default();
        let data = StoreData {
            notes,
            next_sort_order: 0,
        };
        if let Ok(json) = serde_json::to_string(&data) {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(STORAGE_KEY, &json);
                }
            }
        }
    }

    fn load_from_storage(inner: &mut MemoryStore) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(Some(storage)) = window.local_storage() else {
            return;
        };
        let Ok(Some(json)) = storage.get_item(STORAGE_KEY) else {
            return;
        };
        let Ok(data) = serde_json::from_str::<StoreData>(&json) else {
            return;
        };
        for note in data.notes {
            inner.import_note(note);
        }
    }
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
    set_field(&obj, "noteType", &JsValue::from_str(note.note_type.as_str()))?;
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
