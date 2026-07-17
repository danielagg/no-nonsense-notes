use super::*;

impl MemoryStore {
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
