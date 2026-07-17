use super::*;

impl<'a> NoteRepository<'a> {
    pub fn list_add_item(&self, id: NoteId, item: &str) -> Result<Note, StorageError> {
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

        self.conn.execute(
            "UPDATE notes SET title = ?1, content_plaintext = ?2, content_loro_blob = ?3, content_hash = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, plaintext, loro_blob, content_hash, now.to_rfc3339(), id.to_string()],
        )?;

        if let Some(rowid) = self.get_rowid(id)? {
            self.sync_fts(rowid, &title, &plaintext)?;
        }

        self.get(id)
    }

    pub fn list_replace_items(
        &self,
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

        self.conn.execute(
            "UPDATE notes SET title = ?1, content_plaintext = ?2, content_loro_blob = ?3, content_hash = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, plaintext, loro_blob, content_hash, now.to_rfc3339(), id.to_string()],
        )?;

        if let Some(rowid) = self.get_rowid(id)? {
            self.sync_fts(rowid, &title, &plaintext)?;
        }

        self.get(id)
    }

    pub fn list_remove_item(&self, id: NoteId, item: &str) -> Result<Note, StorageError> {
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

        self.conn.execute(
            "UPDATE notes SET title = ?1, content_plaintext = ?2, content_loro_blob = ?3, content_hash = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, plaintext, loro_blob, content_hash, now.to_rfc3339(), id.to_string()],
        )?;

        if let Some(rowid) = self.get_rowid(id)? {
            self.sync_fts(rowid, &title, &plaintext)?;
        }

        self.get(id)
    }
}
