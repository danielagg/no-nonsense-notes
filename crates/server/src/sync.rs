use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use tokio::sync::broadcast;

use crate::AppState;

#[derive(Clone, Debug)]
struct UpdateNotification {
    account_id: String,
    global_seq: i64,
}

#[derive(Clone)]
pub struct SyncHub {
    updates: broadcast::Sender<UpdateNotification>,
}

impl SyncHub {
    pub fn new() -> Self {
        let (updates, _) = broadcast::channel(1024);
        Self { updates }
    }

    fn subscribe(&self) -> broadcast::Receiver<UpdateNotification> {
        self.updates.subscribe()
    }

    fn notify(&self, account_id: &str, global_seq: i64) {
        let _ = self.updates.send(UpdateNotification {
            account_id: account_id.to_owned(),
            global_seq,
        });
    }
}

impl Default for SyncHub {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket sync endpoint.
///
/// ## Protocol
///
/// 1. Connect to `ws://localhost:3000/sync`
/// 2. Send auth token as first text message
/// 3. Server responds with `"unauthorized"` and closes on failure
/// 4. Server responds with `"ready"` on successful authentication
/// 5. Once authenticated, exchange messages:
///    - **Push** (client→server): `[version:1][type:1][doc_id:16][device_id:16][blob_len:4][blob:N]`
///    - **Pull** (client→server): text `"pull:<last_seq>"`
///    - **Push response**: `[global_seq:8]` (little-endian)
///    - **Pull response**: text `seq:<current_seq>\n<doc_id>:<base64_blob>\n...`
///    - **Update notification**: text `update:<global_seq>`; clients then pull
#[utoipa::path(
    get,
    path = "/sync",
    tag = "sync",
    responses(
        (status = 101, description = "WebSocket upgrade successful"),
        (status = 401, description = "Invalid or missing auth token")
    )
)]
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| socket::handle_socket(socket, state))
}

mod messages;
mod socket;
