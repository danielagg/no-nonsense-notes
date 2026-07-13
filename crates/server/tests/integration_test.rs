use futures::{SinkExt, StreamExt};
use no_nonsense_notes_server::storage::Database;
use no_nonsense_notes_server::AppState;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;

// Helper to create a test database
fn test_db() -> Arc<Database> {
    Arc::new(Database::open_in_memory().unwrap())
}

#[tokio::test]
async fn test_signup_and_signin() {
    let db = test_db();

    let app = axum::Router::new()
        .route(
            "/auth/signup",
            axum::routing::post(no_nonsense_notes_server::auth::signup),
        )
        .route(
            "/auth/signin",
            axum::routing::post(no_nonsense_notes_server::auth::signin),
        )
        .with_state(db);

    let client = reqwest::Client::new();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Signup
    let resp = client
        .post(format!("{}/auth/signup", base))
        .json(&serde_json::json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["token"].is_string());
    assert!(body["account_id"].is_string());

    // Signup duplicate email
    let resp = client
        .post(format!("{}/auth/signup", base))
        .json(&serde_json::json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);

    // Signin
    let resp = client
        .post(format!("{}/auth/signin", base))
        .json(&serde_json::json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["token"].is_string());

    // Signin wrong password
    let resp = client
        .post(format!("{}/auth/signin", base))
        .json(&serde_json::json!({
            "email": "test@example.com",
            "password": "wrongpassword"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Signin nonexistent email
    let resp = client
        .post(format!("{}/auth/signin", base))
        .json(&serde_json::json!({
            "email": "nonexistent@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_auth_token_verification() {
    let db = test_db();

    // Insert account and token directly
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

    // Valid token
    let result = no_nonsense_notes_server::auth::verify_token(&db, "valid-token");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "acc1");

    // Invalid token
    let result = no_nonsense_notes_server::auth::verify_token(&db, "invalid-token");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ws_auth_rejection() {
    let db = test_db();

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

    // Send invalid token
    write.send(Message::Text("bad-token".into())).await.unwrap();

    // Should get "unauthorized" response
    let msg = read.next().await.unwrap().unwrap();
    match msg {
        Message::Text(text) => assert_eq!(text.as_str(), "unauthorized"),
        _ => panic!("expected text message"),
    }
}

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
