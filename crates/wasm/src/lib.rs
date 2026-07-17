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

mod js;
mod storage;
mod store;
mod wire;

use js::*;
use storage::*;
pub use wire::*;

#[wasm_bindgen(start)]
pub fn init() {
    web_sys::console::log_1(&"no-nonsense-notes-wasm loaded".into());
}

#[cfg(test)]
mod tests;
