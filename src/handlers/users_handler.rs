use crate::filters;
use crate::models::user_model::{GetUser, LoginUser, UpsertUser};
use crate::{models::user_model::User, AppState};
use askama::Template;
use axum::response::{Html, IntoResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

#[axum::debug_handler]
pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    // Json(User::get(state.pool, id).await)
    let user = User::get(state.pool, id).await;

    user.last_login;

    #[derive(Template)]
    #[template(path = "users/user.j2")]
    struct Template {
        title: String,
        user: GetUser,
    }

    let templ: Template = Template {
        title: user.username.to_string(),
        user,
    };

    Html(templ.render().unwrap())
}

#[axum::debug_handler]
pub async fn all(State(state): State<AppState>) -> impl IntoResponse {
    let users = User::all(state.pool).await;
    // Json(User::all(state.pool).await)

    #[derive(Template)]
    #[template(path = "users/users.j2")]
    struct Template<'a> {
        title: &'a str,
        users: Vec<GetUser>,
    }

    let templ = Template {
        title: "Users",
        users,
    };

    Html(templ.render().unwrap())
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
