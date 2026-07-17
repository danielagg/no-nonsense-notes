use super::super::super::*;

#[wasm_bindgen]
impl WasmStore {
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
