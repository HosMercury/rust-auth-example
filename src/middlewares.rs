use axum::{
    extract::Request,
    http::Method,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::handlers::user_handler::SignInData;

pub async fn auth_middleware(session: Session, request: Request, next: Next) -> Response {
    match session.get::<SignInData>("user").unwrap() {
        Some(_) => next.run(request).await,
        None => {
            if (request.method() == Method::GET && request.uri().path() == "/signin")
                || (request.method() == Method::POST && request.uri().path() == "/signin")
                || (request.method() == Method::GET && request.uri().path() == "/signup")
                || (request.method() == Method::POST && request.uri().path() == "/signup")
                || true
            {
                next.run(request).await
            } else {
                Redirect::to("/signin").into_response()
            }
        }
    }
}
