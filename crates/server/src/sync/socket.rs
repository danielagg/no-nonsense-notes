use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::AppState;
use crate::auth::verify_token;

use super::messages::{handle_binary_message, handle_text_message};

pub(super) async fn handle_socket(mut socket: WebSocket, state: AppState) {
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
