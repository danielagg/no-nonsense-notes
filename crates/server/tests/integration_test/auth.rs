use super::support::test_db;

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
