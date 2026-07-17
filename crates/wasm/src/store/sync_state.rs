use super::super::*;

#[wasm_bindgen]
impl WasmStore {
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
