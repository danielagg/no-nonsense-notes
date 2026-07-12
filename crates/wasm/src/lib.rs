use wasm_bindgen::prelude::*;

use no_nonsense_notes_core::note::{Note, NoteId, NoteType};
use no_nonsense_notes_core::storage::memory::MemoryStore;

type JsResult<T> = Result<T, JsValue>;

#[wasm_bindgen]
pub struct WasmStore {
    inner: MemoryStore,
}

#[wasm_bindgen]
impl WasmStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: MemoryStore::new(),
        }
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

        note_to_js(&note)
    }

    #[wasm_bindgen(js_name = softDelete)]
    pub fn soft_delete(&mut self, id: &str) -> JsResult<()> {
        let id = id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner
            .soft_delete(id)
            .map_err(|e| JsValue::from_str(&e.to_string()))
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
