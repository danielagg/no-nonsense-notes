use super::*;

impl NativeStore {
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
}
