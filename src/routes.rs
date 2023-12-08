use crate::handlers;
use crate::{handlers::main_handler, AppState};
use axum::{routing::get, Router};
use handlers::user_handler;
use tower_http::services::ServeDir;

pub fn web() -> Router<AppState> {
    Router::new()
        .route("/", get(main_handler::index))
        .route(
            "/signup",
            get(user_handler::signup_form).post(user_handler::signup),
        )
        .route("/users", get(user_handler::all))
        .route(
            "/users/:id",
            get(user_handler::get).patch(user_handler::update),
        )
        .nest_service("/public", ServeDir::new("public"))
}
