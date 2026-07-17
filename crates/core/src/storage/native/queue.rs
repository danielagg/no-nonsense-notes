use super::*;

impl NativeStore {
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
    pub(super) fn enqueue(
        &self,
        id: NoteId,
        note_type: Option<NoteType>,
    ) -> Result<(), StorageError> {
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
}
