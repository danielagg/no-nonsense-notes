use super::*;

impl<'a> NoteRepository<'a> {
    pub fn create(
        &self,
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

        self.conn.execute(
            "INSERT INTO notes (id, folder_id, note_type, title, content_plaintext, content_loro_blob, content_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id.to_string(),
                folder_id.map(|f| f.to_string()),
                note_type.as_str(),
                title,
                content_plaintext,
                loro_blob,
                content_hash,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        let rowid = self.conn.last_insert_rowid();
        self.sync_fts(rowid, &title, &content_plaintext)?;

        self.get(id)
    }

    pub fn get(&self, id: NoteId) -> Result<Note, StorageError> {
        let id_str = id.to_string();
        let result = self.conn.query_row(
            &format!("SELECT {SELECT_COLS} FROM notes WHERE id = ?1"),
            params![id_str],
            row_to_note,
        );
        match result {
            Ok(note) => Ok(note),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(StorageError::NotFound { id: id_str }),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update(
        &self,
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

        self.conn.execute(
            "UPDATE notes SET title = ?1, content_plaintext = ?2, content_loro_blob = ?3, content_hash = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, new_content, loro_blob, content_hash, now.to_rfc3339(), id.to_string()],
        )?;

        let rowid = self.get_rowid(id)?;
        if let Some(rid) = rowid {
            self.sync_fts(rid, &title, new_content)?;
        }

        self.get(id)
    }
}
