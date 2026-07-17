use super::*;

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
}

mod notes;
mod sync_state;
