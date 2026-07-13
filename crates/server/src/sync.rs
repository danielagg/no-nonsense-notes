use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use rusqlite::params;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::auth::verify_token;
use crate::{AppState, storage::Database};

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
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // First message must be auth token
    let token = match socket.recv().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        _ => {
            warn!("connection closed before auth");
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };

    let account_id = match verify_token(&state.db, &token) {
        Ok(id) => id,
        Err(_) => {
            warn!("invalid token");
            let _ = socket.send(Message::Text("unauthorized".into())).await;
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };

    info!("client connected: account={}", account_id);

    if socket.send(Message::Text("ready".into())).await.is_err() {
        return;
    }

    let (mut sender, mut receiver) = socket.split();
    let mut updates = state.sync_hub.subscribe();

    loop {
        tokio::select! {
            message = receiver.next() => {
                let Some(message) = message else { break };
                let message = match message {
                    Ok(message) => message,
                    Err(error) => {
                        error!("websocket error: {}", error);
                        break;
                    }
                };

                let result = match message {
                    Message::Binary(data) => {
                        match handle_binary_message(&state.db, &account_id, &data) {
                            Ok((response, global_seq)) => {
                                if let Err(error) = sender.send(Message::Binary(response.into())).await {
                                    Err(anyhow::anyhow!(error))
                                } else {
                                    state.sync_hub.notify(&account_id, global_seq);
                                    Ok(())
                                }
                            }
                            Err(error) => Err(error),
                        }
                    }
                    Message::Text(text) => {
                        match handle_text_message(&state.db, &account_id, &text) {
                            Ok(response) => sender
                                .send(Message::Text(response.into()))
                                .await
                                .map_err(anyhow::Error::from),
                            Err(error) => Err(error),
                        }
                    }
                    Message::Close(_) => break,
                    _ => Ok(()),
                };

                if let Err(error) = result {
                    error!("error handling message: {}", error);
                    break;
                }
            }
            notification = updates.recv() => {
                match notification {
                    Ok(notification) if notification.account_id == account_id => {
                        let message = format!("update:{}", notification.global_seq);
                        if let Err(error) = sender.send(Message::Text(message.into())).await {
                            error!("failed to notify client: {}", error);
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        if let Err(error) = sender.send(Message::Text("update:lagged".into())).await {
                            error!("failed to notify lagged client: {}", error);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    info!("client disconnected: account={}", account_id);
}

/// Binary message format:
/// [version:1][type:1][payload:N]
///
/// Message types:
/// 0x01 = Push updates (client -> server)
/// 0x02 = Pull response (server -> client)
///
/// Push payload:
/// [doc_id:16 bytes UUID][device_id:16 bytes UUID][blob_len:4 bytes LE][blob:N]
///
/// Push response: [global_seq:8 bytes LE]
///
/// Pull request (text for now): "pull:<last_seq>"
/// Pull response payload (text for now):
/// seq:<current_seq>\n<doc_id>:<base64_blob>\n...
const VERSION: u8 = 1;
const MSG_PUSH: u8 = 0x01;

fn handle_binary_message(
    db: &Database,
    account_id: &str,
    data: &[u8],
) -> anyhow::Result<(Vec<u8>, i64)> {
    if data.len() < 2 {
        anyhow::bail!("message too short");
    }

    let version = data[0];
    let msg_type = data[1];

    if version != VERSION {
        anyhow::bail!("unsupported version: {}", version);
    }

    match msg_type {
        MSG_PUSH => handle_push(db, account_id, &data[2..]),
        _ => anyhow::bail!("unknown message type: {}", msg_type),
    }
}

fn handle_push(db: &Database, account_id: &str, payload: &[u8]) -> anyhow::Result<(Vec<u8>, i64)> {
    if payload.len() < 36 {
        anyhow::bail!("push payload too short");
    }

    let doc_id = uuid::Uuid::from_slice(&payload[0..16])?;
    let device_id = uuid::Uuid::from_slice(&payload[16..32])?;
    let blob_len =
        u32::from_le_bytes([payload[32], payload[33], payload[34], payload[35]]) as usize;

    if payload.len() < 36 + blob_len {
        anyhow::bail!("blob length mismatch");
    }

    let blob = &payload[36..36 + blob_len];

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO updates (doc_id, device_id, account_id, blob) VALUES (?1, ?2, ?3, ?4)",
        params![doc_id.to_string(), device_id.to_string(), account_id, blob],
    )?;

    let global_seq: i64 = conn.last_insert_rowid();

    let mut response = Vec::with_capacity(8);
    response.extend_from_slice(&global_seq.to_le_bytes());
    Ok((response, global_seq))
}

fn handle_text_message(db: &Database, account_id: &str, text: &str) -> anyhow::Result<String> {
    let parts: Vec<&str> = text.splitn(2, ':').collect();
    if parts.len() != 2 || parts[0] != "pull" {
        anyhow::bail!("invalid command: {}", text);
    }

    let last_seq: i64 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid seq"))?;

    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT doc_id, global_seq, blob FROM updates WHERE global_seq > ?1 AND account_id = ?2 ORDER BY global_seq ASC LIMIT 1000",
    )?;

    let entries: Vec<(String, i64, Vec<u8>)> = stmt
        .query_map(params![last_seq, account_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let current_seq = entries.iter().map(|e| e.1).max().unwrap_or(last_seq);

    let mut response = format!("seq:{}\n", current_seq);
    for (doc_id, _seq, blob) in &entries {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(blob);
        response.push_str(&format!("{}:{}\n", doc_id, encoded));
    }

    Ok(response)
}
