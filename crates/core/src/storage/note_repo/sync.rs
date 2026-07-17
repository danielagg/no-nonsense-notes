use super::*;

impl<'a> NoteRepository<'a> {
    pub fn apply_remote_update(
        &self,
        note_id: NoteId,
        note_type: NoteType,
        update_blob: &[u8],
    ) -> Result<Note, StorageError> {
        let existing = match self.get(note_id) {
            Ok(note) => Some(note),
            Err(StorageError::NotFound { .. }) => None,
            Err(error) => return Err(error),
        };

        let doc = match existing.as_ref() {
            Some(note) if !note.content_loro_blob.is_empty() => {
                LoroDoc::from_snapshot(&note.content_loro_blob)
                    .map_err(|e| StorageError::Loro(format!("failed to load doc: {e}")))?
            }
            _ => LoroDoc::new(),
        };
        doc.import(update_blob)
            .map_err(|e| StorageError::Loro(format!("failed to import update: {e}")))?;

        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let plaintext = extract_content(&doc, note_type);
        let title = Note::title_from_doc(&doc).unwrap_or_else(|| {
            existing
                .as_ref()
                .map(|note| note.title.clone())
                .unwrap_or_else(|| Note::default_title(note_type).to_string())
        });
        let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
        let now = chrono::Utc::now();

        if existing.is_some() {
            self.conn.execute(
                "UPDATE notes SET note_type = ?1, title = ?2, content_plaintext = ?3, content_loro_blob = ?4, content_hash = ?5, updated_at = ?6, is_deleted = 0, deleted_at = NULL WHERE id = ?7",
                params![note_type.as_str(), title, plaintext, loro_blob, content_hash, now.to_rfc3339(), note_id.to_string()],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO notes (id, note_type, title, content_plaintext, content_loro_blob, content_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![note_id.to_string(), note_type.as_str(), title, plaintext, loro_blob, content_hash, now.to_rfc3339(), now.to_rfc3339()],
            )?;
        }

        if let Some(rowid) = self.get_rowid(note_id)? {
            self.sync_fts(rowid, &title, &plaintext)?;
        }
        self.get(note_id)
    }

    pub fn apply_remote_delete(&self, id: NoteId) -> Result<(), StorageError> {
        if matches!(self.get(id), Err(StorageError::NotFound { .. })) {
            return Ok(());
        }
        self.soft_delete(id)
    }
}
