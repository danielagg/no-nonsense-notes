use super::*;

impl MemoryStore {
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
}
