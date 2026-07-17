use std::collections::HashMap;

use loro::{ExportMode, LoroDoc, LoroValue, ToJson};
use sha2::{Digest, Sha256};

use crate::StorageError;
use crate::note::{Note, NoteId, NoteType};

pub struct MemoryStore {
    notes: HashMap<String, Note>,
    next_sort_order: i64,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            next_sort_order: 0,
        }
    }

    pub fn import_note(&mut self, note: Note) {
        self.notes.insert(note.id.to_string(), note);
    }

    pub fn next_sort_order(&self) -> i64 {
        self.next_sort_order
    }

    pub fn set_next_sort_order(&mut self, val: i64) {
        self.next_sort_order = val;
    }
}

mod lists;
mod notes;
mod query;
mod sync;

#[cfg(test)]
mod tests;

fn list_items_from_doc(doc: &LoroDoc) -> Vec<String> {
    let list = doc.get_list("items");
    list.to_vec()
        .into_iter()
        .map(|v| match v {
            LoroValue::String(s) => s.to_string(),
            other => other.to_json_value().to_string(),
        })
        .collect()
}

fn extract_content(doc: &LoroDoc, note_type: NoteType) -> String {
    match note_type {
        NoteType::Markdown => doc.get_text("content").to_string(),
        NoteType::List => {
            let items = list_items_from_doc(doc);
            items.join("\n")
        }
    }
}
