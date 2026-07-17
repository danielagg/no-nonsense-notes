use super::super::super::*;

#[wasm_bindgen]
impl WasmStore {
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
}
