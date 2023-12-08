use crate::models::user_model::{GetUser, UpsertUser};
use crate::validation::extract_errors;
use crate::{filters, validation};
use crate::{models::user_model::User, AppState};
use askama::Template;
use axum::response::{Html, IntoResponse, Redirect};
use axum::Form;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_flash::{Flash, IncomingFlashes};
use serde::Deserialize;
use uuid::Uuid;
use validation::{validate_password, RE_USERNAME};
use validator::Validate;

#[axum::debug_handler]
pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let user = User::get(state.pool, id).await;

    #[derive(Template)]
    #[template(path = "users/user.html")]
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

    #[derive(Template)]
    #[template(path = "users/users.html")]
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

#[axum::debug_handler]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpsertUser>,
) -> StatusCode {
    User::update(state.pool, payload, id).await;
    StatusCode::CREATED
}

// #[axum::debug_handler]
pub async fn signup_form(
    // State(state): State<AppState>,
    flashes: IncomingFlashes,
) -> impl IntoResponse {
    // let errs: Option<HashMap<String, String>> = session.get("errors").unwrap();

    #[derive(Template)]
    #[template(path = "users/signup.html")]
    struct Template<'a> {
        title: &'a str,
        flash: Option<IncomingFlashes>,
    }

    let templ = Template {
        title: "Sign Up",
        flash: if flashes.is_empty() {
            None
        } else {
            Some(flashes)
        },
    };

    Html(templ.render().unwrap())
}

// post sign-up
#[derive(Deserialize, Validate, Debug)]
pub struct SignUpData {
    #[validate(email(message = "invalid email"))]
    pub email: String,

    #[validate(
        length(min = 4, message = "Username must be greater than 4 chars"),
        regex(
            path = "RE_USERNAME",
            message = "Username must be alphanumeric and/or dashes only"
        )
    )]
    pub username: String,

    #[validate(
        length(min = 4, message = "Password must be greater than 4 chars"),
        custom(
            function = "validate_password",
            message = "password must be 4-50 characters long, contain letters and numbers, and must not contain spaces, special characters, or emoji"
        )
    )]
    pub password: String,

    #[validate(must_match(other = "password", message = "passwords are not identical"))]
    pub password2: String,
}

// #[axum::debug_handler]
pub async fn signup(
    State(state): State<AppState>,
    flash: Flash,
    Form(input): Form<SignUpData>,
) -> (Flash, Redirect) {
    match input.validate() {
        Ok(_) => {
            let user = UpsertUser {
                email: input.email,
                password: input.password,
                username: input.username,
            };

            User::create(state.pool, user).await;

            (flash, Redirect::to("/users"))
        }
        Err(errs) => {
            let errs = extract_errors(errs.errors());

            // let f = errs.iter().map(|(k, v)| flash.error(v)).collect::<Vec<Flash>>();

            let f = errs.iter().fold(flash, |flash, (_, e)| flash.error(e));

            (f, Redirect::to("/signup"))
        }
    }
}
