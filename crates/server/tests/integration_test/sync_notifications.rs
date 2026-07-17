use futures::{SinkExt, StreamExt};
use no_nonsense_notes_server::AppState;
use tokio_tungstenite::tungstenite::Message;

use super::support::test_db;

#[tokio::test]
async fn test_sync_notifies_other_connection() {
    let db = test_db();
    {
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO accounts (id, email, password_hash) VALUES ('acc1', 'test@test.com', 'hash')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO auth_tokens (token, account_id) VALUES ('valid-token', 'acc1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO accounts (id, email, password_hash) VALUES ('acc2', 'other@test.com', 'hash')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO auth_tokens (token, account_id) VALUES ('other-token', 'acc2')",
            [],
        )
        .unwrap();
    }

    let app = axum::Router::new()
        .route(
            "/sync",
            axum::routing::get(no_nonsense_notes_server::sync::ws_handler),
        )
        .with_state(AppState::new(db));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let (first, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/sync"))
        .await
        .unwrap();
    let (second, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/sync"))
        .await
        .unwrap();
    let (other_account, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/sync"))
        .await
        .unwrap();
    let (mut first_write, mut first_read) = first.split();
    let (mut second_write, mut second_read) = second.split();
    let (mut other_write, mut other_read) = other_account.split();

    first_write
        .send(Message::Text("valid-token".into()))
        .await
        .unwrap();
    second_write
        .send(Message::Text("valid-token".into()))
        .await
        .unwrap();
    other_write
        .send(Message::Text("other-token".into()))
        .await
        .unwrap();
    assert!(
        matches!(first_read.next().await.unwrap().unwrap(), Message::Text(text) if text == "ready")
    );
    assert!(
        matches!(second_read.next().await.unwrap().unwrap(), Message::Text(text) if text == "ready")
    );
    assert!(
        matches!(other_read.next().await.unwrap().unwrap(), Message::Text(text) if text == "ready")
    );

    let doc_id = uuid::Uuid::new_v4();
    let device_id = uuid::Uuid::new_v4();
    let blob = [0u8, 1, 2, 3];
    let mut message = vec![1u8, 1u8];
    message.extend_from_slice(doc_id.as_bytes());
    message.extend_from_slice(device_id.as_bytes());
    message.extend_from_slice(&(blob.len() as u32).to_le_bytes());
    message.extend_from_slice(&blob);
    first_write
        .send(Message::Binary(message.into()))
        .await
        .unwrap();

    assert!(matches!(
        first_read.next().await.unwrap().unwrap(),
        Message::Binary(_)
    ));
    let notification = tokio::time::timeout(std::time::Duration::from_secs(1), second_read.next())
        .await
        .expect("second connection was not notified")
        .unwrap()
        .unwrap();
    assert!(matches!(notification, Message::Text(text) if text == "update:1"));
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(150), other_read.next())
            .await
            .is_err()
    );
}
