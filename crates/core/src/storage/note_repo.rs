use loro::{ExportMode, LoroDoc, LoroValue, ToJson};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

use crate::StorageError;
use crate::note::{Note, NoteId, NoteType};

pub struct NoteRepository<'a> {
    conn: &'a Connection,
}

const SELECT_COLS: &str = "id, folder_id, note_type, title, content_plaintext, content_loro_blob, content_hash, created_at, updated_at, is_deleted, deleted_at, sort_order";

const SELECT_COLS_N: &str = "n.id, n.folder_id, n.note_type, n.title, n.content_plaintext, n.content_loro_blob, n.content_hash, n.created_at, n.updated_at, n.is_deleted, n.deleted_at, n.sort_order";

impl<'a> NoteRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(
        &self,
        note_type: NoteType,
        folder_id: Option<NoteId>,
    ) -> Result<Note, StorageError> {
        let id = NoteId::now_v7();
        let now = chrono::Utc::now();

        let doc = LoroDoc::new();
        match note_type {
            NoteType::Markdown => {
                doc.get_text("content");
            }
            NoteType::List => {
                doc.get_list("items");
            }
        }
        let content_plaintext = String::new();
        let content_hash = Sha256::digest(content_plaintext.as_bytes()).to_vec();
        let title = Note::default_title(note_type).to_string();
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to initialize note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;

        self.conn.execute(
            "INSERT INTO notes (id, folder_id, note_type, title, content_plaintext, content_loro_blob, content_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id.to_string(),
                folder_id.map(|f| f.to_string()),
                note_type.as_str(),
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
        let result = self.conn.query_row(
            &format!("SELECT {SELECT_COLS} FROM notes WHERE id = ?1"),
            params![id_str],
            row_to_note,
        );
        match result {
            Ok(note) => Ok(note),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(StorageError::NotFound { id: id_str }),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update(
        &self,
        id: NoteId,
        new_content: &str,
        title_override: Option<&str>,
    ) -> Result<Note, StorageError> {
        let existing = self.get(id)?;
        if existing.note_type != NoteType::Markdown {
            return Err(StorageError::WrongNoteType {
                expected: "markdown".to_string(),
                actual: existing.note_type.as_str().to_string(),
            });
        }

        let doc = if existing.content_loro_blob.is_empty() {
            LoroDoc::new()
        } else {
            LoroDoc::from_snapshot(&existing.content_loro_blob)
                .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?
        };

        let text = doc.get_text("content");
        text.update(new_content, Default::default())
            .map_err(|e| StorageError::Loro(format!("failed to update Loro doc: {e}")))?;

        let content_hash = Sha256::digest(new_content.as_bytes()).to_vec();
        let title = title_override
            .map(|title| Note::normalize_title(existing.note_type, title))
            .unwrap_or_else(|| existing.title.clone());
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to update note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
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

    pub fn list_add_item(&self, id: NoteId, item: &str) -> Result<Note, StorageError> {
        let existing = self.get(id)?;
        if existing.note_type != NoteType::List {
            return Err(StorageError::WrongNoteType {
                expected: "list".to_string(),
                actual: existing.note_type.as_str().to_string(),
            });
        }

        let doc = LoroDoc::from_snapshot(&existing.content_loro_blob)
            .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?;
        let list = doc.get_list("items");
        list.push(item)
            .map_err(|e| StorageError::Loro(format!("failed to push list item: {e}")))?;

        let items = list_items_from_doc(&doc);
        let plaintext = items.join("\n");
        let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
        let title = existing.title.clone();
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to preserve note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let now = chrono::Utc::now();

        self.conn.execute(
            "UPDATE notes SET title = ?1, content_plaintext = ?2, content_loro_blob = ?3, content_hash = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, plaintext, loro_blob, content_hash, now.to_rfc3339(), id.to_string()],
        )?;

        if let Some(rowid) = self.get_rowid(id)? {
            self.sync_fts(rowid, &title, &plaintext)?;
        }

        self.get(id)
    }

    pub fn list_replace_items(
        &self,
        id: NoteId,
        new_items: &[String],
        title_override: Option<&str>,
    ) -> Result<Note, StorageError> {
        let existing = self.get(id)?;
        if existing.note_type != NoteType::List {
            return Err(StorageError::WrongNoteType {
                expected: "list".to_string(),
                actual: existing.note_type.as_str().to_string(),
            });
        }

        let doc = LoroDoc::from_snapshot(&existing.content_loro_blob)
            .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?;
        let list = doc.get_list("items");

        let current_len = list.len();
        if current_len > 0 {
            list.delete(0, current_len)
                .map_err(|e| StorageError::Loro(format!("failed to clear list: {e}")))?;
        }
        for item in new_items {
            list.push(item.as_str())
                .map_err(|e| StorageError::Loro(format!("failed to push list item: {e}")))?;
        }

        let plaintext = new_items.join("\n");
        let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
        let title = title_override
            .map(|title| Note::normalize_title(existing.note_type, title))
            .unwrap_or_else(|| existing.title.clone());
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to update note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let now = chrono::Utc::now();

        self.conn.execute(
            "UPDATE notes SET title = ?1, content_plaintext = ?2, content_loro_blob = ?3, content_hash = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, plaintext, loro_blob, content_hash, now.to_rfc3339(), id.to_string()],
        )?;

        if let Some(rowid) = self.get_rowid(id)? {
            self.sync_fts(rowid, &title, &plaintext)?;
        }

        self.get(id)
    }

    pub fn list_remove_item(&self, id: NoteId, item: &str) -> Result<Note, StorageError> {
        let existing = self.get(id)?;
        if existing.note_type != NoteType::List {
            return Err(StorageError::WrongNoteType {
                expected: "list".to_string(),
                actual: existing.note_type.as_str().to_string(),
            });
        }

        let doc = LoroDoc::from_snapshot(&existing.content_loro_blob)
            .map_err(|e| StorageError::Loro(format!("failed to load Loro doc: {e}")))?;
        let list = doc.get_list("items");

        let pos = list_items_from_doc(&doc)
            .iter()
            .position(|v| v == item)
            .ok_or_else(|| StorageError::NotFound {
                id: format!("list item: {item}"),
            })?;
        list.delete(pos, 1)
            .map_err(|e| StorageError::Loro(format!("failed to delete list item: {e}")))?;

        let items = list_items_from_doc(&doc);
        let plaintext = items.join("\n");
        let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
        let title = existing.title.clone();
        Note::set_title_in_doc(&doc, &title)
            .map_err(|e| StorageError::Loro(format!("failed to preserve note title: {e}")))?;
        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let now = chrono::Utc::now();

        self.conn.execute(
            "UPDATE notes SET title = ?1, content_plaintext = ?2, content_loro_blob = ?3, content_hash = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, plaintext, loro_blob, content_hash, now.to_rfc3339(), id.to_string()],
        )?;

        if let Some(rowid) = self.get_rowid(id)? {
            self.sync_fts(rowid, &title, &plaintext)?;
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

    pub fn apply_remote_update(
        &self,
        note_id: NoteId,
        note_type: NoteType,
        update_blob: &[u8],
    ) -> Result<Note, StorageError> {
        let existing = match self.get(note_id) {
            Ok(note) => Some(note),
            Err(StorageError::NotFound { .. }) => None,
            Err(error) => return Err(error),
        };

        let doc = match existing.as_ref() {
            Some(note) if !note.content_loro_blob.is_empty() => {
                LoroDoc::from_snapshot(&note.content_loro_blob)
                    .map_err(|e| StorageError::Loro(format!("failed to load doc: {e}")))?
            }
            _ => LoroDoc::new(),
        };
        doc.import(update_blob)
            .map_err(|e| StorageError::Loro(format!("failed to import update: {e}")))?;

        let loro_blob = doc
            .export(ExportMode::Snapshot)
            .map_err(|e| StorageError::Loro(e.to_string()))?;
        let plaintext = extract_content(&doc, note_type);
        let title = Note::title_from_doc(&doc).unwrap_or_else(|| {
            existing
                .as_ref()
                .map(|note| note.title.clone())
                .unwrap_or_else(|| Note::default_title(note_type).to_string())
        });
        let content_hash = Sha256::digest(plaintext.as_bytes()).to_vec();
        let now = chrono::Utc::now();

        if existing.is_some() {
            self.conn.execute(
                "UPDATE notes SET note_type = ?1, title = ?2, content_plaintext = ?3, content_loro_blob = ?4, content_hash = ?5, updated_at = ?6, is_deleted = 0, deleted_at = NULL WHERE id = ?7",
                params![note_type.as_str(), title, plaintext, loro_blob, content_hash, now.to_rfc3339(), note_id.to_string()],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO notes (id, note_type, title, content_plaintext, content_loro_blob, content_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![note_id.to_string(), note_type.as_str(), title, plaintext, loro_blob, content_hash, now.to_rfc3339(), now.to_rfc3339()],
            )?;
        }

        if let Some(rowid) = self.get_rowid(note_id)? {
            self.sync_fts(rowid, &title, &plaintext)?;
        }
        self.get(note_id)
    }

    pub fn apply_remote_delete(&self, id: NoteId) -> Result<(), StorageError> {
        if matches!(self.get(id), Err(StorageError::NotFound { .. })) {
            return Ok(());
        }
        self.soft_delete(id)
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

fn list_items_from_doc(doc: &LoroDoc) -> Vec<String> {
    let list = doc.get_list("items");
    list.to_vec()
        .into_iter()
        .map(|v| match v {
            LoroValue::String(s) => s.to_string(),
            other => other.to_json_value().to_string(),
        })
        .collect()
}

fn extract_content(doc: &LoroDoc, note_type: NoteType) -> String {
    match note_type {
        NoteType::Markdown => doc.get_text("content").to_string(),
        NoteType::List => list_items_from_doc(doc).join("\n"),
    }
}

fn row_to_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    let folder_id: Option<String> = row.get(1)?;
    let note_type_str: String = row.get(2)?;
    let is_deleted: bool = row.get(9)?;
    let deleted_at: Option<String> = row.get(10)?;

    let note_type: NoteType = note_type_str.parse().map_err(|e| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(crate::StorageError::Parse(e)))
    })?;

    Ok(Note {
        id: row
            .get::<_, String>(0)?
            .parse()
            .map_err(|e: uuid::Error| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        folder_id: folder_id
            .map(|s| s.parse())
            .transpose()
            .map_err(|e: uuid::Error| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        note_type,
        title: row.get(3)?,
        content_plaintext: row.get(4)?,
        content_loro_blob: row.get(5)?,
        content_hash: row.get(6)?,
        created_at: row
            .get::<_, String>(7)?
            .parse()
            .map_err(|e: chrono::ParseError| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(e))
            })?,
        updated_at: row
            .get::<_, String>(8)?
            .parse()
            .map_err(|e: chrono::ParseError| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(e))
            })?,
        is_deleted,
        deleted_at: deleted_at.map(|s| s.parse()).transpose().map_err(
            |e: chrono::ParseError| rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
        )?,
        sort_order: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::Database;

    #[test]
    fn create_markdown_and_get() {
        let db = Database::open_in_memory().unwrap();
        let repo = NoteRepository::new(db.connection());

        let note = repo.create(NoteType::Markdown, None).unwrap();
        assert_eq!(note.note_type, NoteType::Markdown);
        assert_eq!(note.title, "Untitled");
        assert!(!note.content_loro_blob.is_empty());

        let fetched = repo.get(note.id).unwrap();
        assert_eq!(fetched.id, note.id);
        assert_eq!(fetched.title, "Untitled");
        assert_eq!(fetched.note_type, NoteType::Markdown);
    }

    #[test]
    fn create_list_and_get() {
        let db = Database::open_in_memory().unwrap();
        let repo = NoteRepository::new(db.connection());

        let note = repo.create(NoteType::List, None).unwrap();
        assert_eq!(note.note_type, NoteType::List);
        assert_eq!(note.title, "List");
        assert!(note.content_plaintext.is_empty());

        let fetched = repo.get(note.id).unwrap();
        assert_eq!(fetched.note_type, NoteType::List);
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
            params![
                folder_id.to_string(),
                "Work",
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .unwrap();

        let note = repo.create(NoteType::Markdown, Some(folder_id)).unwrap();
        assert_eq!(note.folder_id, Some(folder_id));
    }

    #[test]
    fn content_does_not_derive_title() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::Markdown, None).unwrap();
        let updated = repo
            .update(note.id, "# Meeting Notes\n\nLorum ipsum.", None)
            .unwrap();
        assert_eq!(updated.title, "Untitled");
    }

    #[test]
    fn update_content_round_trip() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::Markdown, None).unwrap();
        let content = "# Hello\n\nThis is **bold** and *italic*.";
        let updated = repo.update(note.id, content, None).unwrap();
        assert_eq!(updated.content_plaintext, content);
        assert_eq!(updated.title, "Untitled");

        let doc = LoroDoc::from_snapshot(&updated.content_loro_blob).unwrap();
        assert_eq!(doc.get_text("content").to_string(), content);
    }

    #[test]
    fn update_rejects_list_type() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::List, None).unwrap();
        let err = repo.update(note.id, "# Hello", None).unwrap_err();
        assert!(matches!(err, StorageError::WrongNoteType { .. }));
    }

    #[test]
    fn update_with_title_override() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::Markdown, None).unwrap();
        let updated = repo
            .update(note.id, "# Hello", Some("Custom Title"))
            .unwrap();
        assert_eq!(updated.title, "Custom Title");

        let updated = repo.update(note.id, "# Different heading", None).unwrap();
        assert_eq!(updated.title, "Custom Title");
        let doc = LoroDoc::from_snapshot(&updated.content_loro_blob).unwrap();
        assert_eq!(Note::title_from_doc(&doc).as_deref(), Some("Custom Title"));
    }

    #[test]
    fn update_with_empty_title_uses_neutral_default() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::Markdown, None).unwrap();
        let updated = repo.update(note.id, "# Hello", Some("  ")).unwrap();
        assert_eq!(updated.title, "Untitled");
    }

    #[test]
    fn list_replace_items_with_title_override() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::List, None).unwrap();
        let items = vec!["milk".to_string(), "eggs".to_string()];
        let updated = repo
            .list_replace_items(note.id, &items, Some("Shopping"))
            .unwrap();
        assert_eq!(updated.title, "Shopping");
    }

    #[test]
    fn list_add_and_remove() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::List, None).unwrap();
        let id = note.id;
        repo.list_replace_items(id, &[], Some("Shopping")).unwrap();

        repo.list_add_item(id, "milk").unwrap();
        repo.list_add_item(id, "eggs").unwrap();
        repo.list_add_item(id, "bread").unwrap();

        let note = repo.get(id).unwrap();
        assert_eq!(note.content_plaintext, "milk\neggs\nbread");
        assert_eq!(note.title, "Shopping");

        let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
        let list = doc.get_list("items");
        assert_eq!(list.len(), 3);

        let note = repo.list_remove_item(id, "eggs").unwrap();
        assert_eq!(note.content_plaintext, "milk\nbread");

        let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
        let list = doc.get_list("items");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn list_replace_items_replaces_all() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::List, None).unwrap();
        repo.list_add_item(note.id, "milk").unwrap();
        repo.list_add_item(note.id, "eggs").unwrap();

        let new_items = vec!["coffee".to_string(), "sugar".to_string()];
        let note = repo.list_replace_items(note.id, &new_items, None).unwrap();
        assert_eq!(note.content_plaintext, "coffee\nsugar");
        assert_eq!(note.title, "List");

        let results = repo.search("coffee").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn list_replace_items_rejects_markdown() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::Markdown, None).unwrap();
        let err = repo
            .list_replace_items(note.id, &["x".to_string()], None)
            .unwrap_err();
        assert!(matches!(err, StorageError::WrongNoteType { .. }));
    }

    #[test]
    fn list_replace_items_empty_clears_list() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::List, None).unwrap();
        repo.list_add_item(note.id, "milk").unwrap();
        repo.list_add_item(note.id, "eggs").unwrap();

        let note = repo.list_replace_items(note.id, &[], None).unwrap();
        assert_eq!(note.content_plaintext, "");
        assert_eq!(note.title, "List");

        let doc = LoroDoc::from_snapshot(&note.content_loro_blob).unwrap();
        let list = doc.get_list("items");
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn list_remove_missing_errors() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::List, None).unwrap();
        repo.list_add_item(note.id, "milk").unwrap();

        let err = repo.list_remove_item(note.id, "nope").unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[test]
    fn list_add_rejects_markdown() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::Markdown, None).unwrap();
        let err = repo.list_add_item(note.id, "milk").unwrap_err();
        assert!(matches!(err, StorageError::WrongNoteType { .. }));
    }

    #[test]
    fn list_items_searchable() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let n1 = repo.create(NoteType::List, None).unwrap();
        repo.list_add_item(n1.id, "milk").unwrap();
        repo.list_add_item(n1.id, "eggs").unwrap();

        let n2 = repo.create(NoteType::Markdown, None).unwrap();
        repo.update(n2.id, "Meeting with Alice", None).unwrap();

        let results = repo.search("milk").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, n1.id);
        assert_eq!(results[0].note_type, NoteType::List);
    }

    #[test]
    fn soft_delete_hides_note() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let note = repo.create(NoteType::Markdown, None).unwrap();
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

        repo.create(NoteType::Markdown, None).unwrap();
        repo.create(NoteType::List, None).unwrap();
        repo.create(NoteType::Markdown, None).unwrap();

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
            params![
                folder_id.to_string(),
                "Work",
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .unwrap();

        repo.create(NoteType::Markdown, None).unwrap();
        repo.create(NoteType::List, Some(folder_id)).unwrap();
        repo.create(NoteType::Markdown, Some(folder_id)).unwrap();

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

        let n1 = repo.create(NoteType::Markdown, None).unwrap();
        repo.update(n1.id, "Groceries: milk and eggs", None)
            .unwrap();
        let n2 = repo.create(NoteType::Markdown, None).unwrap();
        repo.update(n2.id, "Meeting with Alice", None).unwrap();

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

        let note = repo.create(NoteType::Markdown, None).unwrap();
        repo.update(note.id, "Important stuff", None).unwrap();
        repo.soft_delete(note.id).unwrap();

        let results = repo.search("Important").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn get_missing_returns_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::storage::migrations::run(&conn).unwrap();
        let repo = NoteRepository::new(&conn);

        let id = NoteId::now_v7();
        let err = repo.get(id).unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }
}
