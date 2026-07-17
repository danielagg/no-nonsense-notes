use super::*;

impl WasmStore {
    pub(crate) fn storage_key(&self, base: &str) -> String {
        format!("{base}:{}", self.storage_namespace)
    }

    pub(crate) fn save_to_storage(&self) {
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

    pub(crate) fn load_from_storage(inner: &mut MemoryStore, storage_namespace: &str) {
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

pub(crate) fn get_local_storage_item(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

pub(crate) fn set_local_storage_item(key: &str, value: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}

pub(crate) fn remove_local_storage_item(key: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item(key);
        }
    }
}
