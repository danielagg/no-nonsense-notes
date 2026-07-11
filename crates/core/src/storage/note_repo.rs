use loro::{ExportMode, LoroDoc};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

use crate::note::{Note, NoteId};
use crate::StorageError;

pub struct NoteRepository<'a> {
    conn: &'a Connection,
}

impl<'a> NoteRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, folder_id: Option<NoteId>) -> Result<Note, StorageError> {
        let id = NoteId::now_v7();
        let now = chrono::Utc::now();

        let doc = LoroDoc::new();
        doc.get_text("content");
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;

        let content_plaintext = String::new();
        let content_hash = Sha256::digest(content_plaintext.as_bytes()).to_vec();
        let title = Note::derive_title(&content_plaintext);

        self.conn.execute(
            "INSERT INTO notes (id, folder_id, title, content_plaintext, content_loro_blob, content_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id.to_string(),
                folder_id.map(|f| f.to_string()),
                title,
                content_plaintext,
                loro_blob,
                content_hash,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        let rowid = self.conn.last_insert_rowid();
        self.sync_fts(rowid, &title, &content_plaintext)?;

        self.get(id)
    }

    pub fn get(&self, id: NoteId) -> Result<Note, StorageError> {
        let id_str = id.to_string();
        self.conn
            .query_row(
                "SELECT id, folder_id, title, content_plaintext, content_loro_blob, content_hash, created_at, updated_at, is_deleted, deleted_at, sort_order FROM notes WHERE id = ?1",
                params![id_str],
                |row| {
                    let folder_id: Option<String> = row.get(1)?;
                    let is_deleted: bool = row.get(8)?;
                    let deleted_at: Option<String> = row.get(9)?;

                    Ok(Note {
                        id: row.get::<_, String>(0)?.parse().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                        folder_id: folder_id
                            .map(|s| s.parse())
                            .transpose()
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                        title: row.get(2)?,
                        content_plaintext: row.get(3)?,
                        content_loro_blob: row.get(4)?,
                        content_hash: row.get(5)?,
                        created_at: row.get::<_, String>(6)?.parse().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                        updated_at: row.get::<_, String>(7)?.parse().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                        is_deleted,
                        deleted_at: deleted_at
                            .map(|s| s.parse())
                            .transpose()
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                        sort_order: row.get(10)?,
                    })
                },
            )
            .map_err(StorageError::from)
    }

    pub fn update(&self, id: NoteId, new_content: &str) -> Result<Note, StorageError> {
        let existing = self.get(id)?;

        let doc = if existing.content_loro_blob.is_empty() {
            LoroDoc::new()
        } else {
            LoroDoc::from_snapshot(&existing.content_loro_blob)
                .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?
        };

        let text = doc.get_text("content");
        text.update(new_content, Default::default())
            .map_err(|e| StorageError::Loro(format!("failed to update Loro doc: {e}")))?;

        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let content_hash = Sha256::digest(new_content.as_bytes()).to_vec();
        let title = Note::derive_title(new_content);
        let now = chrono::Utc::now();

        self.conn.execute(
            "UPDATE notes SET title = ?1, content_plaintext = ?2, content_loro_blob = ?3, content_hash = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, new_content, loro_blob, content_hash, now.to_rfc3339(), id.to_string()],
        )?;

        let rowid = self.get_rowid(id)?;
        if let Some(rid) = rowid {
            self.sync_fts(rid, &title, new_content)?;
        }

        self.get(id)
    }

    pub fn soft_delete(&self, id: NoteId) -> Result<(), StorageError> {
        let now = chrono::Utc::now();
        self.conn.execute(
            "UPDATE notes SET is_deleted = 1, deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![now.to_rfc3339(), now.to_rfc3339(), id.to_string()],
        )?;

        if let Some(rowid) = self.get_rowid(id)? {
            self.conn.execute("DELETE FROM notes_fts WHERE rowid = ?1", params![rowid])?;
        }

        Ok(())
    }

    pub fn list(&self, folder_id: Option<NoteId>) -> Result<Vec<Note>, StorageError> {
        let sql = "SELECT id, folder_id, title, content_plaintext, content_loro_blob, content_hash, created_at, updated_at, is_deleted, deleted_at, sort_order \
                   FROM notes WHERE is_deleted = 0 \
                   AND (?1 IS NULL OR folder_id = ?1) \
                   ORDER BY updated_at DESC";

        let mut stmt = self.conn.prepare(sql)?;
        let folder_str = folder_id.map(|f| f.to_string());
        let rows = stmt.query_map(params![folder_str], row_to_note)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::from)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Note>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.folder_id, n.title, n.content_plaintext, n.content_loro_blob, n.content_hash, n.created_at, n.updated_at, n.is_deleted, n.deleted_at, n.sort_order \
             FROM notes_fts \
             JOIN notes n ON notes_fts.rowid = n.rowid \
             WHERE notes_fts MATCH ?1 AND n.is_deleted = 0 \
             ORDER BY rank",
        )?;

        let rows = stmt.query_map(params![query], row_to_note)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::from)
    }

    fn sync_fts(&self, rowid: i64, title: &str, content: &str) -> Result<(), StorageError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO notes_fts (rowid, title, content_plaintext) VALUES (?1, ?2, ?3)",
            params![rowid, title, content],
        )?;
        Ok(())
    }

    fn get_rowid(&self, id: NoteId) -> Result<Option<i64>, StorageError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::Database;

    #[test]
    fn create_and_get() {
        let db = Database::open_in_memory().unwrap();
        let repo = NoteRepository::new(db.connection());

        let note = repo.create(None).unwrap();
        assert_eq!(note.title, "Untitled");
        assert!(!note.content_loro_blob.is_empty());

        let fetched = repo.get(note.id).unwrap();
        assert_eq!(fetched.id, note.id);
        assert_eq!(fetched.title, "Untitled");
    }

    #[test]
    fn create_with_folder() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let folder_id = NoteId::now_v7();
        conn.execute(
            "INSERT INTO folders (id, name, sort_order, created_at) VALUES (?1, ?2, 0, ?3)",
            params![folder_id.to_string(), "Work", chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();

        let note = repo.create(Some(folder_id)).unwrap();
        assert_eq!(note.folder_id, Some(folder_id));
    }

    #[test]
    fn derive_title_from_content() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(None).unwrap();
        let updated = repo
            .update(note.id, "# Meeting Notes\n\nLorum ipsum.")
            .unwrap();
        assert_eq!(updated.title, "Meeting Notes");
    }

    #[test]
    fn update_content_round_trip() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(None).unwrap();
        let content = "# Hello\n\nThis is **bold** and *italic*.";
        let updated = repo.update(note.id, content).unwrap();
        assert_eq!(updated.content_plaintext, content);
        assert_eq!(updated.title, "Hello");

        // Verify Loro doc loads from blob and content matches
        let doc = LoroDoc::from_snapshot(&updated.content_loro_blob).unwrap();
        assert_eq!(doc.get_text("content").to_string(), content);
    }

    #[test]
    fn soft_delete_hides_note() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(None).unwrap();
        repo.soft_delete(note.id).unwrap();

        let list = repo.list(None).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn list_notes() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        repo.create(None).unwrap();
        repo.create(None).unwrap();
        repo.create(None).unwrap();

        let list = repo.list(None).unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn list_filtered_by_folder() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let folder_id = NoteId::now_v7();
        conn.execute(
            "INSERT INTO folders (id, name, sort_order, created_at) VALUES (?1, ?2, 0, ?3)",
            params![folder_id.to_string(), "Work", chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();

        repo.create(None).unwrap();
        repo.create(Some(folder_id)).unwrap();
        repo.create(Some(folder_id)).unwrap();

        let all = repo.list(None).unwrap();
        assert_eq!(all.len(), 3);

        let filtered = repo.list(Some(folder_id)).unwrap();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn search_notes() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let n1 = repo.create(None).unwrap();
        repo.update(n1.id, "Groceries: milk and eggs").unwrap();
        let n2 = repo.create(None).unwrap();
        repo.update(n2.id, "Meeting with Alice").unwrap();

        let results = repo.search("Groceries").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, n1.id);
    }

    #[test]
    fn search_ignores_deleted() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(None).unwrap();
        repo.update(note.id, "Important stuff").unwrap();
        repo.soft_delete(note.id).unwrap();

        let results = repo.search("Important").unwrap();
        assert!(results.is_empty());
    }
}

fn row_to_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    let folder_id: Option<String> = row.get(1)?;
    let is_deleted: bool = row.get(8)?;
    let deleted_at: Option<String> = row.get(9)?;

    Ok(Note {
        id: row.get::<_, String>(0)?.parse().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        folder_id: folder_id.map(|s| s.parse()).transpose().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        title: row.get(2)?,
        content_plaintext: row.get(3)?,
        content_loro_blob: row.get(4)?,
        content_hash: row.get(5)?,
        created_at: row.get::<_, String>(6)?.parse().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        updated_at: row.get::<_, String>(7)?.parse().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        is_deleted,
        deleted_at: deleted_at.map(|s| s.parse()).transpose().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        sort_order: row.get(10)?,
    })
}
