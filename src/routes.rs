use crate::AppState;
use axum::{
    routing::{get, patch, post},
    Router,
};
use handlers::users_handler;

use crate::handlers;

pub fn web() -> Router<AppState> {
    Router::new()
        .route("/users", get(users_handler::all))
        .route("/users/:id", get(users_handler::get))
        .route("/users/create", post(users_handler::create))
        .route("/users/:id", patch(users_handler::update))
}
