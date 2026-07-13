pub mod auth;
pub mod error;
pub mod storage;
pub mod sync;

use std::sync::Arc;

use axum::extract::FromRef;

use storage::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub sync_hub: sync::SyncHub,
}

impl AppState {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            sync_hub: sync::SyncHub::new(),
        }
    }
}

impl FromRef<AppState> for Arc<Database> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
