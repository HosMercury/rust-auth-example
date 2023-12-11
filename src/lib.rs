mod filters;
mod handlers;
mod helpers;
mod middlewares;
mod models;
mod routes;
mod validation;

use axum::{error_handling::HandleErrorLayer, extract::FromRef, http::StatusCode, BoxError};
use axum_flash::Key;
use once_cell::sync::Lazy;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, fs, io};
use time::Duration;
use tower::ServiceBuilder;
use tower_sessions::{fred::prelude::*, Expiry, RedisStore, SessionManagerLayer};

struct Manifest {
    css_link: String,
    js_link: String,
}

static MANIFEST: Lazy<Manifest> = Lazy::new(|| {
    let files = fs::read_dir("./public")
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    let mut css_link: String = "".to_string();
    let mut js_link: String = "".to_string();

    for file in files.into_iter() {
        let extension = file.as_path().extension().and_then(|ext| ext.to_str());
        let f = file.as_path().display().to_string();

        match extension {
            Some("js") => js_link = f,
            Some("css") => css_link = f,
            _ => (),
        }
    }

    Manifest { css_link, js_link }
});

// static ENV_PRODUCTION: Lazy<bool> = Lazy::new(|| {
//     let env: String = env::var("ENV").unwrap();
//     env == "production"
// });

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub pool: Pool<Postgres>,
    pub flash_config: axum_flash::Config,
}

impl FromRef<AppState> for axum_flash::Config {
    fn from_ref(state: &AppState) -> axum_flash::Config {
        state.flash_config.clone()
    }
}

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Lazy::force(&MANIFEST);
    // tracing_subscriber::fmt::init();

    dotenvy::dotenv().expect("error loading .env");
    let db_url: String = env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("error connection to db");

    ////////////////// SESSION //////////////////////////////
    let client = RedisClient::default();
    let redis_conn = client.connect();
    client.wait_for_connect().await?;

    let session_store = RedisStore::new(client);
    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::seconds(60 * 60 * 24 * 30))),
        );

    // state
    let state = AppState {
        app_name: "Rust Auth".to_string(),
        pool,
        flash_config: axum_flash::Config::new(Key::generate()),
    };

    let app = routes::web().with_state(state).layer(session_service);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8899").await?;
    // tracing::debug!("listening on {:?}", listener);
    // println!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    redis_conn.await??;
    Ok(())
}
