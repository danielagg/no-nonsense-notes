use no_nonsense_notes_core::note::{Note, NoteId, NoteType};

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

pub(crate) fn parse_id(id: &str) -> Result<NoteId, NativeError> {
    id.parse()
        .map_err(|error: uuid::Error| NativeError::Core(error.to_string()))
}

pub(crate) fn to_record(note: Note) -> NoteRecord {
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
