use axum::{extract::State, response::Html, response::IntoResponse, routing::get, Json, Router};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::net::SocketAddr;
use std::sync::Arc;

mod handler;
mod model;
mod schema;

pub struct AppState {
    database_url: String,
    db: MySqlPool,
}

#[tokio::main]
async fn main() {
    // 設定値の取得
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // DBの作成
    let pool = match MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let state = AppState {
        database_url,
        db: pool,
    };
    let shared_state = Arc::new(state);

    // アプリ部
    // let app = Router::new().route("/", get(handler));
    let app = Router::new()
        .route("/", get(health_checker_handler))
        .route("/hello", get(handler))
        .route("/hello/url", get(url_handler))
        .route("/api/healthchecker", get(health_checker_handler))
        .with_state(shared_state);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn url_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let html_string: String = format!(
        "<h1>Hello, World!</h1>\
        <p>DatabeseUrl : {:?}</p>\
    ",
        &state.as_ref().database_url
    );
    Html(html_string)
}

async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Rust CRUD API Example with Axum Framework and MySQL";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}
