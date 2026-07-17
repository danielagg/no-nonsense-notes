use super::*;

impl MemoryStore {
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
}
