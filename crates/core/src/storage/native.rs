use std::path::Path;
use std::sync::Mutex;

use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use crate::StorageError;
use crate::note::{Note, NoteId, NoteType};
use crate::storage::note_repo::NoteRepository;
use crate::storage::sqlite::Database;
use crate::sync::protocol::{self, PullPayload, PushPayload};

#[derive(Debug, Clone)]
pub struct PendingPush {
    pub doc_id: NoteId,
    pub note_type: Option<NoteType>,
    pub generation: i64,
}

pub struct NativeStore {
    database: Mutex<Database>,
}

impl NativeStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        Ok(Self {
            database: Mutex::new(Database::open(path.as_ref())?),
        })
    }

    fn with_repo<T>(
        &self,
        operation: impl FnOnce(&NoteRepository<'_>) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        operation(&NoteRepository::new(database.connection()))
    }
}

mod notes;
mod pull;
mod queue;
mod settings;
