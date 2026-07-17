use super::super::*;

#[wasm_bindgen]
impl WasmStore {
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
}

mod lists;
mod query;
mod remote;
