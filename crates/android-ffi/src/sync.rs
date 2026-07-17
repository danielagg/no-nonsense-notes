use std::sync::Arc;

use no_nonsense_notes_core::sync::client::{self, SyncClient};

use crate::{NotesStore, SyncStatus};

#[uniffi::export(with_foreign)]
pub trait SyncDelegate: Send + Sync {
    fn state_changed(&self, status: SyncStatus, detail: Option<String>);
    fn notes_changed(&self);
}

struct DelegateAdapter {
    delegate: Arc<dyn SyncDelegate>,
}

impl client::SyncDelegate for DelegateAdapter {
    fn state_changed(&self, state: client::SyncState, detail: Option<String>) {
        self.delegate.state_changed(state.into(), detail);
    }
    fn notes_changed(&self) {
        self.delegate.notes_changed();
    }
}

#[derive(uniffi::Object)]
pub struct SyncSession {
    inner: SyncClient,
}

#[uniffi::export]
impl SyncSession {
    #[uniffi::constructor]
    pub fn start(
        store: Arc<NotesStore>,
        websocket_url: String,
        token: String,
        delegate: Arc<dyn SyncDelegate>,
    ) -> Arc<Self> {
        let adapter = Arc::new(DelegateAdapter { delegate });
        Arc::new(Self {
            inner: SyncClient::start(websocket_url, token, store.inner.clone(), adapter),
        })
    }

    pub fn wake(&self) {
        self.inner.wake();
    }
    pub fn stop(&self) {
        self.inner.stop();
    }
}

impl From<client::SyncState> for SyncStatus {
    fn from(value: client::SyncState) -> Self {
        match value {
            client::SyncState::Disconnected => Self::Disconnected,
            client::SyncState::Connecting => Self::Connecting,
            client::SyncState::Connected => Self::Connected,
            client::SyncState::Error => Self::Error,
        }
    }
}
