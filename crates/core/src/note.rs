use serde::{Deserialize, Serialize};

pub type NoteId = uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteType {
    Markdown,
    List,
}

impl NoteType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::List => "list",
        }
    }
}

impl std::fmt::Display for NoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for NoteType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "markdown" => Ok(Self::Markdown),
            "list" => Ok(Self::List),
            other => Err(format!("unknown note_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub folder_id: Option<NoteId>,
    pub note_type: NoteType,
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