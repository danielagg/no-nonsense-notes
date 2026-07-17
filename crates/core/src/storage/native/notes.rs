use super::*;

impl NativeStore {
    pub fn create(&self, note_type: NoteType) -> Result<Note, StorageError> {
        let note = self.with_repo(|repo| repo.create(note_type, None))?;
        self.enqueue(note.id, Some(note_type))?;
        Ok(note)
    }

    pub fn get(&self, id: NoteId) -> Result<Note, StorageError> {
        self.with_repo(|repo| repo.get(id))
    }

    pub fn list(&self) -> Result<Vec<Note>, StorageError> {
        self.with_repo(|repo| repo.list(None))
    }

    pub fn search(&self, query: &str) -> Result<Vec<Note>, StorageError> {
        if query.trim().is_empty() {
            return self.list();
        }
        self.with_repo(|repo| repo.search(query))
    }

    pub fn update_markdown(
        &self,
        id: NoteId,
        content: &str,
        title: Option<&str>,
    ) -> Result<Note, StorageError> {
        let note = self.with_repo(|repo| repo.update(id, content, title))?;
        self.enqueue(id, Some(NoteType::Markdown))?;
        Ok(note)
    }

    pub fn update_list(
        &self,
        id: NoteId,
        items: &[String],
        title: Option<&str>,
    ) -> Result<Note, StorageError> {
        let note = self.with_repo(|repo| repo.list_replace_items(id, items, title))?;
        self.enqueue(id, Some(NoteType::List))?;
        Ok(note)
    }

    pub fn delete(&self, id: NoteId) -> Result<(), StorageError> {
        self.with_repo(|repo| repo.soft_delete(id))?;
        self.enqueue(id, None)
    }
}
