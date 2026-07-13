use loro::{LoroDoc, LoroValue};
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

    pub fn as_byte(self) -> u8 {
        match self {
            Self::Markdown => 0,
            Self::List => 1,
        }
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Markdown),
            1 => Some(Self::List),
            _ => None,
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
    pub fn default_title(note_type: NoteType) -> &'static str {
        match note_type {
            NoteType::Markdown => "Untitled",
            NoteType::List => "List",
        }
    }

    pub fn normalize_title(note_type: NoteType, title: &str) -> String {
        let title = title.trim();
        if title.is_empty() {
            Self::default_title(note_type).to_string()
        } else {
            title.to_string()
        }
    }

    pub fn set_title_in_doc(doc: &LoroDoc, title: &str) -> loro::LoroResult<()> {
        doc.get_map("metadata").insert("title", title)
    }

    pub fn title_from_doc(doc: &LoroDoc) -> Option<String> {
        let value = doc.try_get_map("metadata")?.get("title")?.get_deep_value();
        match value {
            LoroValue::String(title) => Some(title.to_string()),
            _ => None,
        }
    }
}
