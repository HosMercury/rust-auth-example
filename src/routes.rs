use crate::{handlers::main_handler, AppState};
use axum::{
    routing::{get, patch, post},
    Router,
};
use handlers::users_handler;
use tower_http::services::ServeDir;

use crate::handlers;

pub fn web() -> Router<AppState> {
    let service = ServeDir::new("public");

    Router::new()
        .route("/", get(main_handler::index))
        .route("/users", get(users_handler::all))
        .route("/users/:id", get(users_handler::get))
        .route("/users/:id", patch(users_handler::update))
        .route("/users/create", post(users_handler::create))
        .route("/users/login", post(users_handler::login))
        .nest_service("/public", ServeDir::new("public"))
}
