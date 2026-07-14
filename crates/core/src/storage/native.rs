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

    pub fn cursor(&self) -> Result<i64, StorageError> {
        Ok(self
            .setting("sync_cursor")?
            .and_then(|value| value.parse().ok())
            .unwrap_or(0))
    }

    pub fn device_id(&self) -> Result<Uuid, StorageError> {
        if let Some(value) = self.setting("device_id")? {
            return value
                .parse()
                .map_err(|e: uuid::Error| StorageError::Parse(e.to_string()));
        }
        let id = Uuid::now_v7();
        self.set_setting("device_id", &id.to_string())?;
        Ok(id)
    }

    pub fn next_pending(&self) -> Result<Option<PendingPush>, StorageError> {
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        let row = database
            .connection()
            .query_row(
                "SELECT doc_id, note_type, generation FROM pending_sync ORDER BY rowid LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()?;
        row.map(|(id, kind, generation)| {
            let doc_id = id
                .parse()
                .map_err(|e: uuid::Error| StorageError::Parse(e.to_string()))?;
            let note_type = if kind == "delete" {
                None
            } else {
                Some(kind.parse().map_err(StorageError::Parse)?)
            };
            Ok(PendingPush {
                doc_id,
                note_type,
                generation,
            })
        })
        .transpose()
    }

    pub fn frame_for(&self, pending: &PendingPush) -> Result<Vec<u8>, StorageError> {
        let device_id = self.device_id()?;
        match pending.note_type {
            Some(note_type) => {
                let note = self.get(pending.doc_id)?;
                Ok(protocol::encode_push_frame(&PushPayload {
                    doc_id: pending.doc_id,
                    device_id,
                    note_type,
                    loro_blob: &note.content_loro_blob,
                }))
            }
            None => Ok(protocol::encode_delete_frame(pending.doc_id, device_id)),
        }
    }

    pub fn acknowledge(&self, pending: &PendingPush) -> Result<(), StorageError> {
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        database.connection().execute(
            "DELETE FROM pending_sync WHERE doc_id = ?1 AND generation = ?2",
            params![pending.doc_id.to_string(), pending.generation],
        )?;
        Ok(())
    }

    pub fn apply_pull_response(&self, text: &str) -> Result<usize, StorageError> {
        let response = protocol::decode_pull_response(text).map_err(StorageError::Protocol)?;
        let count = response.entries.len();
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        let repo = NoteRepository::new(database.connection());
        for entry in response.entries {
            match entry.payload {
                PullPayload::Note {
                    note_type,
                    loro_blob,
                } => {
                    repo.apply_remote_update(entry.doc_id, note_type, &loro_blob)?;
                }
                PullPayload::Tombstone => repo.apply_remote_delete(entry.doc_id)?,
            }
        }
        database.connection().execute(
            "INSERT INTO settings(key, value) VALUES('sync_cursor', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![response.current_seq.to_string()],
        )?;
        Ok(count)
    }

    fn enqueue(&self, id: NoteId, note_type: Option<NoteType>) -> Result<(), StorageError> {
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        database.connection().execute(
            "INSERT INTO pending_sync(doc_id, note_type, generation) VALUES(?1, ?2, 1) ON CONFLICT(doc_id) DO UPDATE SET note_type = excluded.note_type, generation = pending_sync.generation + 1",
            params![id.to_string(), note_type.map(|kind| kind.as_str()).unwrap_or("delete")],
        )?;
        Ok(())
    }

    fn setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        Ok(database
            .connection()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?)
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let database = self
            .database
            .lock()
            .map_err(|_| StorageError::Protocol("database lock poisoned".into()))?;
        database.connection().execute(
            "INSERT INTO settings(key, value) VALUES(?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}
