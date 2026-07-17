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
}

mod lists;
mod notes;
mod query;
mod sync;

#[cfg(test)]
mod tests;

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
