mod filters;
mod handlers;
mod models;
mod routes;

use std::net::SocketAddr;
use std::{env, fs, io};

use once_cell::sync::Lazy;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

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

    let mut css_link: String = "nothing".to_string();
    let mut js_link: String = "nothing".to_string();

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
}

#[tokio::main]
pub async fn run() {
    Lazy::force(&MANIFEST);

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
