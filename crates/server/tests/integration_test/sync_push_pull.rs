use futures::{SinkExt, StreamExt};
use no_nonsense_notes_server::AppState;
use tokio_tungstenite::tungstenite::Message;

use super::support::test_db;

#[tokio::test]
async fn test_sync_push_and_pull() {
    let db = test_db();

    // Insert a valid token
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

    let (ws_stream, _) = tokio_tungstenite::connect_async(format!("ws://{}/sync", addr))
        .await
        .unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Authenticate
    write
        .send(Message::Text("valid-token".into()))
        .await
        .unwrap();
    assert!(matches!(
        read.next().await.unwrap().unwrap(),
        Message::Text(text) if text == "ready"
    ));

    // Push a binary update
    let doc_id = uuid::Uuid::new_v4();
    let device_id = uuid::Uuid::new_v4();
    let blob = vec![1u8, 2, 3, 4, 5];

    let mut payload = Vec::new();
    payload.extend_from_slice(doc_id.as_bytes());
    payload.extend_from_slice(device_id.as_bytes());
    payload.extend_from_slice(&(blob.len() as u32).to_le_bytes());
    payload.extend_from_slice(&blob);

    let mut msg = Vec::new();
    msg.push(1u8); // version
    msg.push(1u8); // MSG_PUSH
    msg.extend_from_slice(&payload);

    write.send(Message::Binary(msg.into())).await.unwrap();

    // Read push response (global_seq)
    let response = read.next().await.unwrap().unwrap();
    match response {
        Message::Binary(data) => {
            assert_eq!(data.len(), 8);
            let seq = i64::from_le_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ]);
            assert_eq!(seq, 1); // First entry
        }
        _ => panic!("expected binary response"),
    }

    // Pull updates
    write.send(Message::Text("pull:0".into())).await.unwrap();

    let response = loop {
        let response = read.next().await.unwrap().unwrap();
        if matches!(&response, Message::Text(text) if text.starts_with("seq:")) {
            break response;
        }
    };
    match response {
        Message::Text(text) => {
            assert!(text.starts_with("seq:1\n"));
            assert!(text.contains(&doc_id.to_string()));
        }
        _ => panic!("expected text response"),
    }

    // A deletion is an opaque one-byte 0xFF tombstone and must survive
    // server storage/pull unchanged.
    let tombstone = [0xFF];
    let mut msg = Vec::new();
    msg.push(1u8);
    msg.push(1u8);
    msg.extend_from_slice(doc_id.as_bytes());
    msg.extend_from_slice(device_id.as_bytes());
    msg.extend_from_slice(&(tombstone.len() as u32).to_le_bytes());
    msg.extend_from_slice(&tombstone);
    write.send(Message::Binary(msg.into())).await.unwrap();

    loop {
        if matches!(read.next().await.unwrap().unwrap(), Message::Binary(_)) {
            break;
        }
    }

    write.send(Message::Text("pull:1".into())).await.unwrap();
    let response = loop {
        let response = read.next().await.unwrap().unwrap();
        if matches!(&response, Message::Text(text) if text.starts_with("seq:")) {
            break response;
        }
    };
    match response {
        Message::Text(text) => {
            assert!(text.starts_with("seq:2\n"));
            assert!(text.contains(&format!("{doc_id}:/w==")));
        }
        _ => panic!("expected text response"),
    }
}
