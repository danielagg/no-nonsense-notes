use std::collections::HashMap;

use loro::{ExportMode, LoroDoc, LoroValue, ToJson};
use sha2::{Digest, Sha256};

use crate::note::{Note, NoteId, NoteType};
use crate::StorageError;

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

    pub fn create(
        &mut self,
        note_type: NoteType,
        folder_id: Option<NoteId>,
    ) -> Result<Note, StorageError> {
        let id = NoteId::now_v7();
        let now = chrono::Utc::now();

        let doc = LoroDoc::new();
        match note_type {
            NoteType::Markdown => {
                doc.get_text("content");
            }
            NoteType::List => {
                doc.get_list("items");
            }
        }
        let content_plaintext = String::new();
        let content_hash = Sha256::digest(content_plaintext.as_bytes()).to_vec();
        let title = Note::default_title(note_type).to_string();
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to initialize note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;

        let sort_order = self.next_sort_order;
        self.next_sort_order += 1;

        let note = Note {
            id,
            folder_id,
            note_type,
            title,
            content_plaintext,
            content_loro_blob: loro_blob,
            content_hash,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            deleted_at: None,
            sort_order,
        };

        self.notes.insert(id.to_string(), note.clone());
        Ok(note)
    }

    pub fn get(&self, id: NoteId) -> Result<Note, StorageError> {
        self.notes
            .get(&id.to_string())
            .cloned()
            .ok_or_else(|| StorageError::NotFound { id: id.to_string() })
    }

    pub fn update(
        &mut self,
        id: NoteId,
        new_content: &str,
        title_override: Option<&str>,
    ) -> Result<Note, StorageError> {
        let existing = self.get(id)?;
        if existing.note_type != NoteType::Markdown {
            return Err(StorageError::WrongNoteType {
                expected: "markdown".to_string(),
                actual: existing.note_type.as_str().to_string(),
            });
        }

        let doc = if existing.content_loro_blob.is_empty() {
            LoroDoc::new()
        } else {
            LoroDoc::from_snapshot(&existing.content_loro_blob)
                .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?
        };

        let text = doc.get_text("content");
        text.update(new_content, Default::default())
            .map_err(|e| StorageError::Loro(format!("failed to update Loro doc: {e}")))?;

        let content_hash = Sha256::digest(new_content.as_bytes()).to_vec();
        let title = title_override
            .map(|title| Note::normalize_title(existing.note_type, title))
            .unwrap_or_else(|| existing.title.clone());
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to update note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let now = chrono::Utc::now();

        let mut note = existing;
        note.title = title;
        note.content_plaintext = new_content.to_string();
        note.content_loro_blob = loro_blob;
        note.content_hash = content_hash;
        note.updated_at = now;

        self.notes.insert(id.to_string(), note.clone());
        Ok(note)
    }

    pub fn list_add_item(&mut self, id: NoteId, item: &str) -> Result<Note, StorageError> {
        let existing = self.get(id)?;
        if existing.note_type != NoteType::List {
            return Err(StorageError::WrongNoteType {
                expected: "list".to_string(),
                actual: existing.note_type.as_str().to_string(),
            });
        }

        let doc = LoroDoc::from_snapshot(&existing.content_loro_blob)
            .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?;
        let list = doc.get_list("items");
        list.push(item)
            .map_err(|e| StorageError::Loro(format!("failed to push list item: {e}")))?;

        let items = list_items_from_doc(&doc);
        let plaintext = items.join("\n");
        let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
        let title = existing.title.clone();
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to preserve note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let now = chrono::Utc::now();

        let mut note = existing;
        note.title = title;
        note.content_plaintext = plaintext;
        note.content_loro_blob = loro_blob;
        note.content_hash = content_hash;
        note.updated_at = now;

        self.notes.insert(id.to_string(), note.clone());
        Ok(note)
    }

    pub fn list_replace_items(
        &mut self,
        id: NoteId,
        new_items: &[String],
        title_override: Option<&str>,
    ) -> Result<Note, StorageError> {
        let existing = self.get(id)?;
        if existing.note_type != NoteType::List {
            return Err(StorageError::WrongNoteType {
                expected: "list".to_string(),
                actual: existing.note_type.as_str().to_string(),
            });
        }

        let doc = LoroDoc::from_snapshot(&existing.content_loro_blob)
            .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?;
        let list = doc.get_list("items");

        let current_len = list.len();
        if current_len > 0 {
            list.delete(0, current_len)
                .map_err(|e| StorageError::Loro(format!("failed to clear list: {e}")))?;
        }
        for item in new_items {
            list.push(item.as_str())
                .map_err(|e| StorageError::Loro(format!("failed to push list item: {e}")))?;
        }

        let plaintext = new_items.join("\n");
        let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
        let title = title_override
            .map(|title| Note::normalize_title(existing.note_type, title))
            .unwrap_or_else(|| existing.title.clone());
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to update note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let now = chrono::Utc::now();

        let mut note = existing;
        note.title = title;
        note.content_plaintext = plaintext;
        note.content_loro_blob = loro_blob;
        note.content_hash = content_hash;
        note.updated_at = now;

        self.notes.insert(id.to_string(), note.clone());
        Ok(note)
    }

    pub fn list_remove_item(&mut self, id: NoteId, item: &str) -> Result<Note, StorageError> {
        let existing = self.get(id)?;
        if existing.note_type != NoteType::List {
            return Err(StorageError::WrongNoteType {
                expected: "list".to_string(),
                actual: existing.note_type.as_str().to_string(),
            });
        }

        let doc = LoroDoc::from_snapshot(&existing.content_loro_blob)
            .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?;
        let list = doc.get_list("items");

        let pos = list_items_from_doc(&doc)
            .iter()
            .position(|v| v == item)
            .ok_or_else(|| StorageError::NotFound {
                id: format!("list item: {item}"),
            })?;
        list.delete(pos, 1)
            .map_err(|e| StorageError::Loro(format!("failed to delete list item: {e}")))?;

        let items = list_items_from_doc(&doc);
        let plaintext = items.join("\n");
        let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
        let title = existing.title.clone();
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to preserve note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let now = chrono::Utc::now();

        let mut note = existing;
        note.title = title;
        note.content_plaintext = plaintext;
        note.content_loro_blob = loro_blob;
        note.content_hash = content_hash;
        note.updated_at = now;

        self.notes.insert(id.to_string(), note.clone());
        Ok(note)
    }

    pub fn apply_remote_update(
        &mut self,
        note_id: NoteId,
        note_type: NoteType,
        update_blob: &[u8],
    ) -> Result<Note, StorageError> {
        if let Some(existing) = self.notes.get(&note_id.to_string()).cloned() {
            let doc = if existing.content_loro_blob.is_empty() {
                LoroDoc::new()
            } else {
                LoroDoc::from_snapshot(&existing.content_loro_blob)
                    .map_err(|e| StorageError::Loro(format!("failed to load doc: {e}")))?
            };
            doc.import(update_blob)
                .map_err(|e| StorageError::Loro(format!("failed to import update: {e}")))?;

            let loro_blob = doc
                .export(ExportMode::Snapshot)
                .map_err(|e| StorageError::Loro(e.to_string()))?;

            let plaintext = extract_content(&doc, note_type);
            let title = Note::title_from_doc(&doc).unwrap_or_else(|| existing.title.clone());
            let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
            let now = chrono::Utc::now();

            let mut note = existing;
            note.note_type = note_type;
            note.title = title;
            note.content_plaintext = plaintext;
            note.content_loro_blob = loro_blob;
            note.content_hash = content_hash;
            note.updated_at = now;

            self.notes.insert(note_id.to_string(), note.clone());
            Ok(note)
        } else {
            let doc = LoroDoc::new();
            doc.import(update_blob)
                .map_err(|e| StorageError::Loro(format!("failed to import update: {e}")))?;

            let loro_blob = doc
                .export(ExportMode::Snapshot)
                .map_err(|e| StorageError::Loro(e.to_string()))?;

            let plaintext = extract_content(&doc, note_type);
            let title = Note::title_from_doc(&doc)
                .unwrap_or_else(|| Note::default_title(note_type).to_string());
            let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
            let now = chrono::Utc::now();
            let sort_order = self.next_sort_order;
            self.next_sort_order += 1;

            let note = Note {
                id: note_id,
                folder_id: None,
                note_type,
                title,
                content_plaintext: plaintext,
                content_loro_blob: loro_blob,
                content_hash,
                created_at: now,
                updated_at: now,
                is_deleted: false,
                deleted_at: None,
                sort_order,
            };

            self.notes.insert(note_id.to_string(), note.clone());
            Ok(note)
        }
    }

    pub fn soft_delete(&mut self, id: NoteId) -> Result<(), StorageError> {
        let mut existing = self.get(id)?;
        let now = chrono::Utc::now();
        existing.is_deleted = true;
        existing.deleted_at = Some(now);
        existing.updated_at = now;
        self.notes.insert(id.to_string(), existing);
        Ok(())
    }

    /// Applies a deletion received from another device. Missing notes are already
    /// deleted from this store's point of view, so tombstone replay is idempotent.
    pub fn apply_remote_delete(&mut self, id: NoteId) {
        if let Some(existing) = self.notes.get_mut(&id.to_string()) {
            if existing.is_deleted {
                return;
            }
            let now = chrono::Utc::now();
            existing.is_deleted = true;
            existing.deleted_at = Some(now);
            existing.updated_at = now;
        }
    }

    pub fn list(&self, folder_id: Option<NoteId>) -> Result<Vec<Note>, StorageError> {
        let mut results: Vec<Note> = self
            .notes
            .values()
            .filter(|n| !n.is_deleted)
            .filter(|n| folder_id.is_none() || n.folder_id == folder_id)
            .cloned()
            .collect();
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(results)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Note>, StorageError> {
        let q = query.to_lowercase();
        let mut results: Vec<Note> = self
            .notes
            .values()
            .filter(|n| !n.is_deleted)
            .filter(|n| {
                n.title.to_lowercase().contains(&q)
                    || n.content_plaintext.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(results)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_markdown_and_get() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();
        assert_eq!(note.note_type, NoteType::Markdown);
        assert_eq!(note.title, "Untitled");
        assert!(!note.content_loro_blob.is_empty());

        let fetched = store.get(note.id).unwrap();
        assert_eq!(fetched.id, note.id);
        assert_eq!(fetched.title, "Untitled");
    }

    #[test]
    fn create_list_and_get() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::List, None).unwrap();
        assert_eq!(note.note_type, NoteType::List);
        assert_eq!(note.title, "List");

        let fetched = store.get(note.id).unwrap();
        assert_eq!(fetched.note_type, NoteType::List);
    }

    #[test]
    fn update_content_round_trip() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();
        let content = "# Hello\n\nThis is **bold** and *italic*.";
        let updated = store.update(note.id, content, None).unwrap();
        assert_eq!(updated.content_plaintext, content);
        assert_eq!(updated.title, "Untitled");

        let doc = LoroDoc::from_snapshot(&updated.content_loro_blob).unwrap();
        assert_eq!(doc.get_text("content").to_string(), content);
    }

    #[test]
    fn update_rejects_list_type() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::List, None).unwrap();
        let err = store.update(note.id, "# Hello", None).unwrap_err();
        assert!(matches!(err, StorageError::WrongNoteType { .. }));
    }

    #[test]
    fn update_with_title_override() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();
        let updated = store
            .update(note.id, "# Hello", Some("Custom Title"))
            .unwrap();
        assert_eq!(updated.title, "Custom Title");
    }

    #[test]
    fn update_with_empty_title_uses_neutral_default() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();
        let updated = store.update(note.id, "# Hello", Some("  ")).unwrap();
        assert_eq!(updated.title, "Untitled");
    }

    #[test]
    fn content_update_without_title_preserves_custom_title() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();
        store
            .update(note.id, "# First", Some("User Title"))
            .unwrap();
        let updated = store.update(note.id, "# World", None).unwrap();
        assert_eq!(updated.title, "User Title");
    }

    #[test]
    fn list_replace_items_with_title_override() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::List, None).unwrap();
        let items = vec!["milk".to_string(), "eggs".to_string()];
        let updated = store
            .list_replace_items(note.id, &items, Some("Shopping List"))
            .unwrap();
        assert_eq!(updated.title, "Shopping List");
    }

    #[test]
    fn list_add_and_remove() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::List, None).unwrap();
        let id = note.id;
        store.list_replace_items(id, &[], Some("Shopping")).unwrap();

        store.list_add_item(id, "milk").unwrap();
        store.list_add_item(id, "eggs").unwrap();
        store.list_add_item(id, "bread").unwrap();

        let note = store.get(id).unwrap();
        assert_eq!(note.content_plaintext, "milk\neggs\nbread");
        assert_eq!(note.title, "Shopping");

        let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
        let list = doc.get_list("items");
        assert_eq!(list.len(), 3);

        let note = store.list_remove_item(id, "eggs").unwrap();
        assert_eq!(note.content_plaintext, "milk\nbread");
    }

    #[test]
    fn list_replace_items_replaces_all() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::List, None).unwrap();
        let id = note.id;

        store.list_add_item(id, "milk").unwrap();
        store.list_add_item(id, "eggs").unwrap();

        let new_items = vec![
            "coffee".to_string(),
            "sugar".to_string(),
            "flour".to_string(),
        ];
        let note = store.list_replace_items(id, &new_items, None).unwrap();
        assert_eq!(note.content_plaintext, "coffee\nsugar\nflour");
        assert_eq!(note.title, "List");

        let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
        let list = doc.get_list("items");
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn list_replace_items_rejects_markdown() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();
        let err = store
            .list_replace_items(note.id, &["x".to_string()], None)
            .unwrap_err();
        assert!(matches!(err, StorageError::WrongNoteType { .. }));
    }

    #[test]
    fn list_replace_items_empty_clears_list() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::List, None).unwrap();
        store.list_add_item(note.id, "milk").unwrap();
        store.list_add_item(note.id, "eggs").unwrap();

        let note = store.list_replace_items(note.id, &[], None).unwrap();
        assert_eq!(note.content_plaintext, "");
        assert_eq!(note.title, "List");

        let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
        let list = doc.get_list("items");
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn list_remove_missing_errors() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::List, None).unwrap();
        store.list_add_item(note.id, "milk").unwrap();

        let err = store.list_remove_item(note.id, "nope").unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[test]
    fn soft_delete_hides_note() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();
        store.soft_delete(note.id).unwrap();

        let list = store.list(None).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn remote_delete_is_idempotent_for_existing_and_missing_notes() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();

        store.apply_remote_delete(note.id);
        store.apply_remote_delete(note.id);
        store.apply_remote_delete(NoteId::now_v7());

        assert!(store.list(None).unwrap().is_empty());
        assert!(store.get(note.id).unwrap().is_deleted);
    }

    #[test]
    fn search_notes() {
        let mut store = MemoryStore::new();
        let n1 = store.create(NoteType::Markdown, None).unwrap();
        store
            .update(n1.id, "Groceries: milk and eggs", None)
            .unwrap();
        let n2 = store.create(NoteType::Markdown, None).unwrap();
        store.update(n2.id, "Meeting with Alice", None).unwrap();

        let results = store.search("Groceries").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, n1.id);
    }

    #[test]
    fn get_missing_returns_not_found() {
        let store = MemoryStore::new();
        let id = NoteId::now_v7();
        let err = store.get(id).unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[test]
    fn list_filtered_by_folder() {
        let mut store = MemoryStore::new();
        let folder_id = NoteId::now_v7();

        store.create(NoteType::Markdown, None).unwrap();
        store.create(NoteType::List, Some(folder_id)).unwrap();
        store.create(NoteType::Markdown, Some(folder_id)).unwrap();

        let all = store.list(None).unwrap();
        assert_eq!(all.len(), 3);

        let filtered = store.list(Some(folder_id)).unwrap();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn apply_remote_update_creates_new_note() {
        let mut store = MemoryStore::new();

        let doc = LoroDoc::new();
        let text = doc.get_text("content");
        text.insert(0, "# Remote Note\n\nHello from another device")
            .unwrap();
        Note::set_title_in_doc(&doc, "Remote Note").unwrap();
        let blob = doc.export(ExportMode::Snapshot).unwrap();

        let remote_id = NoteId::now_v7();
        let note = store
            .apply_remote_update(remote_id, NoteType::Markdown, &blob)
            .unwrap();

        assert_eq!(note.id, remote_id);
        assert_eq!(note.note_type, NoteType::Markdown);
        assert_eq!(note.title, "Remote Note");
        assert!(note.content_plaintext.contains("Hello from another device"));

        let fetched = store.get(remote_id).unwrap();
        assert_eq!(fetched.id, remote_id);
    }

    #[test]
    fn apply_remote_update_merges_into_existing() {
        let mut store = MemoryStore::new();
        let note = store.create(NoteType::Markdown, None).unwrap();
        store.update(note.id, "# Original\n\nHello", None).unwrap();

        let doc = LoroDoc::from_snapshot(&store.get(note.id).unwrap().content_loro_blob).unwrap();
        let text = doc.get_text("content");
        let pos = text.to_string().len();
        text.insert(pos, "\n\nAppended by remote").unwrap();
        let update_blob = doc.export(ExportMode::Snapshot).unwrap();

        let merged = store
            .apply_remote_update(note.id, NoteType::Markdown, &update_blob)
            .unwrap();

        assert!(merged.content_plaintext.contains("Hello"));
        assert!(merged.content_plaintext.contains("Appended by remote"));
    }

    #[test]
    fn apply_remote_update_list_note() {
        let mut store = MemoryStore::new();

        let doc = LoroDoc::new();
        let list = doc.get_list("items");
        list.push("milk").unwrap();
        list.push("eggs").unwrap();
        Note::set_title_in_doc(&doc, "Groceries").unwrap();
        let blob = doc.export(ExportMode::Snapshot).unwrap();

        let remote_id = NoteId::now_v7();
        let note = store
            .apply_remote_update(remote_id, NoteType::List, &blob)
            .unwrap();

        assert_eq!(note.note_type, NoteType::List);
        assert_eq!(note.content_plaintext, "milk\neggs");
        assert_eq!(note.title, "Groceries");
    }

    #[test]
    fn title_is_part_of_remote_update_metadata() {
        let mut source = MemoryStore::new();
        let note = source.create(NoteType::Markdown, None).unwrap();
        let renamed = source
            .update(note.id, "# Heading", Some("User-owned title"))
            .unwrap();

        let mut destination = MemoryStore::new();
        let received = destination
            .apply_remote_update(note.id, NoteType::Markdown, &renamed.content_loro_blob)
            .unwrap();
        assert_eq!(received.title, "User-owned title");

        let edited = source.update(note.id, "# Different heading", None).unwrap();
        let received = destination
            .apply_remote_update(note.id, NoteType::Markdown, &edited.content_loro_blob)
            .unwrap();
        assert_eq!(received.title, "User-owned title");
    }
}
