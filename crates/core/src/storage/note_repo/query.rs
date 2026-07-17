use super::*;

impl<'a> NoteRepository<'a> {
    pub fn soft_delete(&self, id: NoteId) -> Result<(), StorageError> {
        let now = chrono::Utc::now();
        self.conn.execute(
            "UPDATE notes SET is_deleted = 1, deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![now.to_rfc3339(), now.to_rfc3339(), id.to_string()],
        )?;

        if let Some(rowid) = self.get_rowid(id)? {
            self.conn
                .execute("DELETE FROM notes_fts WHERE rowid = ?1", params![rowid])?;
        }

        Ok(())
    }

    pub fn list(&self, folder_id: Option<NoteId>) -> Result<Vec<Note>, StorageError> {
        let sql = format!(
            "SELECT {SELECT_COLS} \
               FROM notes WHERE is_deleted = 0 \
               AND (?1 IS NULL OR folder_id = ?1) \
               ORDER BY updated_at DESC"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let folder_str = folder_id.map(|f| f.to_string());
        let rows = stmt.query_map(params![folder_str], row_to_note)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Note>, StorageError> {
        let sql = format!(
            "SELECT {SELECT_COLS_N} \
              FROM notes_fts \
              JOIN notes n ON notes_fts.rowid = n.rowid \
              WHERE notes_fts MATCH ?1 AND n.is_deleted = 0 \
              ORDER BY rank"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![query], row_to_note)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub(super) fn sync_fts(
        &self,
        rowid: i64,
        title: &str,
        content: &str,
    ) -> Result<(), StorageError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO notes_fts (rowid, title, content_plaintext) VALUES (?1, ?2, ?3)",
            params![rowid, title, content],
        )?;
        Ok(())
    }

    pub(super) fn get_rowid(&self, id: NoteId) -> Result<Option<i64>, StorageError> {
        let id_str = id.to_string();
        let result = self.conn.query_row(
            "SELECT rowid FROM notes WHERE id = ?1",
            params![id_str],
            |row| row.get(0),
        );
        match result {
            Ok(rid) => Ok(Some(rid)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
