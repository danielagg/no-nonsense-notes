use std::sync::Arc;

use no_nonsense_notes_core::storage::native::NativeStore;

use crate::records::{NativeError, NoteKind, NoteRecord, parse_id, to_record};

#[derive(uniffi::Object)]
pub struct NotesStore {
    pub(crate) inner: Arc<NativeStore>,
}

#[uniffi::export]
impl NotesStore {
    #[uniffi::constructor]
    pub fn open(database_path: String) -> Result<Arc<Self>, NativeError> {
        Ok(Arc::new(Self {
            inner: Arc::new(NativeStore::open(database_path)?),
        }))
    }

    pub fn create_note(&self, kind: NoteKind) -> Result<NoteRecord, NativeError> {
        Ok(to_record(self.inner.create(kind.into())?))
    }

    pub fn get_note(&self, id: String) -> Result<NoteRecord, NativeError> {
        Ok(to_record(self.inner.get(parse_id(&id)?)?))
    }

    pub fn list_notes(&self) -> Result<Vec<NoteRecord>, NativeError> {
        Ok(self.inner.list()?.into_iter().map(to_record).collect())
    }

    pub fn search_notes(&self, query: String) -> Result<Vec<NoteRecord>, NativeError> {
        Ok(self
            .inner
            .search(&query)?
            .into_iter()
            .map(to_record)
            .collect())
    }

    pub fn update_markdown(
        &self,
        id: String,
        content: String,
        title: Option<String>,
    ) -> Result<NoteRecord, NativeError> {
        Ok(to_record(self.inner.update_markdown(
            parse_id(&id)?,
            &content,
            title.as_deref(),
        )?))
    }

    pub fn update_list(
        &self,
        id: String,
        items: Vec<String>,
        title: Option<String>,
    ) -> Result<NoteRecord, NativeError> {
        Ok(to_record(self.inner.update_list(
            parse_id(&id)?,
            &items,
            title.as_deref(),
        )?))
    }

    pub fn delete_note(&self, id: String) -> Result<(), NativeError> {
        self.inner.delete(parse_id(&id)?)?;
        Ok(())
    }
}
