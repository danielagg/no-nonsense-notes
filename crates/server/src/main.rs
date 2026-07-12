use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use utoipa::{OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use no_nonsense_notes_server::{auth, storage, sync};

#[derive(OpenApi)]
#[openapi(
    paths(auth::signup, auth::signin, sync::ws_handler),
    components(schemas(auth::SignupRequest, auth::SigninRequest, auth::AuthResponse)),
    info(title = "No Nonsense Notes API", version = "0.1.0")
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "server.db".into());
    let db = Arc::new(storage::Database::open(db_path.as_ref()).expect("failed to open database"));

    let cors_origin = std::env::var("CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".into());
    tracing::info!("CORS origin: {}", cors_origin);

    let cors = CorsLayer::new()
        .allow_origin(cors_origin.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any());

    let app = Router::new()
        .route("/auth/signup", post(auth::signup))
        .route("/auth/signin", post(auth::signin))
        .route("/sync", get(sync::ws_handler))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(db);

    let port = std::env::var("PORT")
        .or_else(|_| std::env::var("LISTEN_ADDR"))
        .unwrap_or_else(|_| "3000".into());
    let addr = if port.contains(':') {
        port
    } else {
        format!("0.0.0.0:{}", port)
    };
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
