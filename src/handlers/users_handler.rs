use crate::models::user::{GetUser, LoginUser, UpsertUser};
use crate::{models::user::User, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

// #[axum::debug_handler]
pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Json<GetUser> {
    Json(User::get(state.pool, id).await)
}

// #[axum::debug_handler]
pub async fn all(State(state): State<AppState>) -> Json<Vec<GetUser>> {
    Json(User::all(state.pool).await)
}

// #[axum::debug_handler]
pub async fn create(State(state): State<AppState>, Json(payload): Json<UpsertUser>) -> StatusCode {
    User::create(state.pool, payload).await;
    StatusCode::CREATED
}

// #[axum::debug_handler]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpsertUser>,
) -> StatusCode {
    User::update(state.pool, payload, id).await;
    StatusCode::CREATED
}

// #[axum::debug_handler]
pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginUser>) -> StatusCode {
    User::login(state.pool, payload).await;
    StatusCode::OK
}
