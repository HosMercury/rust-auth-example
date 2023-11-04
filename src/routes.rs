use crate::{AppState, handlers::main_handler};
use axum::{
    routing::{get, patch, post},
    Router,
};
use handlers::users_handler;

use crate::handlers;

pub fn web() -> Router<AppState> {
    Router::new()
        .route("/", get(main_handler::index))
        .route("/users", get(users_handler::all))
        .route("/users/:id", get(users_handler::get))
        .route("/users/:id", patch(users_handler::update))
        .route("/users/create", post(users_handler::create))
        .route("/users/login", post(users_handler::login))
}
