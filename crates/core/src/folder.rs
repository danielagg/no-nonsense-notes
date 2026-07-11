use serde::{Deserialize, Serialize};

pub type FolderId = uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub sort_order: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
