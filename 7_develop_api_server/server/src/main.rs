use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use dotenv::dotenv;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod handler;
mod model;
mod route;
mod schema;

pub struct AppState {
    database_url: String,
    db: MySqlPool,
}

use route::create_router;

#[tokio::main]
async fn main() {
    // 設定値の取得
    dotenv().ok();
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

    // CROSについて調べる
    // [CORS関連、これだけ知っとけばまぁ大丈夫 #xhr - Qiita](https://qiita.com/rooooomania/items/4d0f6275372f413765de#cors%E3%81%A8%E3%83%A6%E3%83%BC%E3%82%B6%E8%AA%8D%E8%A8%BC%E6%83%85%E5%A0%B1)

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(shared_state).layer(cors);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
