use super::super::super::*;

#[wasm_bindgen]
impl WasmStore {
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

    #[wasm_bindgen(js_name = applyRemoteDelete)]
    pub fn apply_remote_delete(&mut self, note_id: &str) -> JsResult<()> {
        let id = note_id
            .parse::<NoteId>()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.apply_remote_delete(id);
        self.save_to_storage();
        Ok(())
    }
}
