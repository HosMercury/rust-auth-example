use crate::handlers;
use crate::middlewares::auth_middleware;
use crate::{handlers::main_handler, AppState};
use axum::middleware;
use axum::{
    routing::{get, post},
    Router,
};
use handlers::user_handler;
use tower_http::services::ServeDir;

pub fn web() -> Router<AppState> {
    Router::new()
        .route("/", get(main_handler::index))
        .route(
            "/signup",
            get(user_handler::signup_form).post(user_handler::signup),
        )
        .route(
            "/signin",
            get(user_handler::signin_form).post(user_handler::signin),
        )
        .route("/signout", post(user_handler::signout))
        .route("/g/callback", get(user_handler::google_oauth_callback))
        .route("/users", get(user_handler::all))
        .route(
            "/users/:id",
            get(user_handler::get).patch(user_handler::update),
        )
        .layer(middleware::from_fn(auth_middleware))
        .nest_service("/public", ServeDir::new("public"))
}
