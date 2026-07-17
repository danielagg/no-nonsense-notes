use futures::{SinkExt, StreamExt};
use no_nonsense_notes_server::AppState;
use tokio_tungstenite::tungstenite::Message;

use super::support::test_db;

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
