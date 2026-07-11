use serde::{Deserialize, Serialize};

pub type NoteId = uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub folder_id: Option<NoteId>,
    pub title: String,
    pub content_plaintext: String,
    pub content_loro_blob: Vec<u8>,
    pub content_hash: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub sort_order: i64,
}

impl Note {
    pub fn derive_title(content: &str) -> String {
        for line in content.lines() {
            if let Some(stripped) = line.strip_prefix("# ") {
                let t = stripped.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
        "Untitled".to_string()
    }
}
