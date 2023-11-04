mod handlers;
mod models;
mod routes;

use std::env;
use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub pool: Pool<Postgres>,
}

#[tokio::main]
pub async fn run() {
    // .env
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().expect("error loading .env");
    let db_url: String = env::var("DATABASE_URL").unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("error connection to db");

    // state
    let state = AppState {
        app_name: "Rust-Press".to_string(),
        pool,
    };
    let app = routes::web().with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
