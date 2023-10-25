use axum::{http::StatusCode, Json};

use crate::models::users::{CreateUser, User};

pub(crate) async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1337,
        username: payload.username,
    };
    (StatusCode::CREATED, Json(user))
}
