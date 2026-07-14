use std::sync::Arc;

use no_nonsense_notes_core::note::{Note, NoteId, NoteType};
use no_nonsense_notes_core::storage::native::NativeStore;
use no_nonsense_notes_core::sync::client::{self, SyncClient};

uniffi::setup_scaffolding!();

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum NoteKind {
    Markdown,
    List,
}

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum SyncStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct NoteRecord {
    pub id: String,
    pub kind: NoteKind,
    pub title: String,
    pub content: String,
    pub items: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum NativeError {
    #[error("{0}")]
    Core(String),
}

impl From<no_nonsense_notes_core::StorageError> for NativeError {
    fn from(value: no_nonsense_notes_core::StorageError) -> Self {
        Self::Core(value.to_string())
    }
}

#[derive(uniffi::Object)]
pub struct NotesStore {
    inner: Arc<NativeStore>,
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

#[uniffi::export(with_foreign)]
pub trait SyncDelegate: Send + Sync {
    fn state_changed(&self, status: SyncStatus, detail: Option<String>);
    fn notes_changed(&self);
}

struct DelegateAdapter {
    delegate: Arc<dyn SyncDelegate>,
}

impl client::SyncDelegate for DelegateAdapter {
    fn state_changed(&self, state: client::SyncState, detail: Option<String>) {
        self.delegate.state_changed(state.into(), detail);
    }
    fn notes_changed(&self) {
        self.delegate.notes_changed();
    }
}

#[derive(uniffi::Object)]
pub struct SyncSession {
    inner: SyncClient,
}

#[uniffi::export]
impl SyncSession {
    #[uniffi::constructor]
    pub fn start(
        store: Arc<NotesStore>,
        websocket_url: String,
        token: String,
        delegate: Arc<dyn SyncDelegate>,
    ) -> Arc<Self> {
        let adapter = Arc::new(DelegateAdapter { delegate });
        Arc::new(Self {
            inner: SyncClient::start(websocket_url, token, store.inner.clone(), adapter),
        })
    }

    pub fn wake(&self) {
        self.inner.wake();
    }
    pub fn stop(&self) {
        self.inner.stop();
    }
}

fn parse_id(id: &str) -> Result<NoteId, NativeError> {
    id.parse()
        .map_err(|error: uuid::Error| NativeError::Core(error.to_string()))
}

fn to_record(note: Note) -> NoteRecord {
    let items = if note.note_type == NoteType::List {
        note.content_plaintext
            .split('\n')
            .map(str::to_owned)
            .filter(|item| !item.is_empty())
            .collect()
    } else {
        Vec::new()
    };
    NoteRecord {
        id: note.id.to_string(),
        kind: note.note_type.into(),
        title: note.title,
        content: note.content_plaintext,
        items,
        created_at: note.created_at.to_rfc3339(),
        updated_at: note.updated_at.to_rfc3339(),
    }
}

impl From<NoteKind> for NoteType {
    fn from(value: NoteKind) -> Self {
        match value {
            NoteKind::Markdown => Self::Markdown,
            NoteKind::List => Self::List,
        }
    }
}

impl From<NoteType> for NoteKind {
    fn from(value: NoteType) -> Self {
        match value {
            NoteType::Markdown => Self::Markdown,
            NoteType::List => Self::List,
        }
    }
}

impl From<client::SyncState> for SyncStatus {
    fn from(value: client::SyncState) -> Self {
        match value {
            client::SyncState::Disconnected => Self::Disconnected,
            client::SyncState::Connecting => Self::Connecting,
            client::SyncState::Connected => Self::Connected,
            client::SyncState::Error => Self::Error,
        }
    }
}
