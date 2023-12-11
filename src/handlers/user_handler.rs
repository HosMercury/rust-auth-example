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
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use uuid::Uuid;
use validation::{email_exists, username_exists, validate_password, RE_USERNAME};
use validator::Validate;

#[axum::debug_handler]
pub async fn get(
    State(state): State<AppState>,
    flash: Flash,
    flashes: IncomingFlashes,
    Path(id): Path<Uuid>,
) -> (Flash, impl IntoResponse) {
    let user = User::get(state.pool, id).await;

    #[derive(Template)]
    #[template(path = "users/user.html")]
    struct Template {
        title: String,
        user: GetUser,
        flash: IncomingFlashes,
    }

    let templ: Template = Template {
        title: user.username.to_string(),
        user,
        flash: flashes,
    };

    (flash, Html(templ.render().unwrap()))
}

// #[axum::debug_handler]
pub async fn all(
    flashes: IncomingFlashes,
    flash: Flash,
    State(state): State<AppState>,
) -> (Flash, impl IntoResponse) {
    let users = User::all(state.pool).await;

    #[derive(Template)]
    #[template(path = "users/users.html")]
    struct Template<'a> {
        title: &'a str,
        users: Vec<GetUser>,
        flash: IncomingFlashes,
    }

    let templ = Template {
        title: "Users",
        users,
        flash: flashes,
    };

    (flash, Html(templ.render().unwrap()))
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
pub async fn signup_form(flash: Flash, flashes: IncomingFlashes) -> (Flash, impl IntoResponse) {
    #[derive(Template)]
    #[template(path = "users/signup.html")]
    struct Template<'a> {
        title: &'a str,
        flash: IncomingFlashes,
    }

    let templ = Template {
        title: "Sign Up",
        flash: flashes,
    };

    (flash, Html(templ.render().unwrap()))
}

// post sign-up
#[derive(Deserialize, Validate, Debug)]
pub struct SignUpData {
    #[validate(
        length(min = 4, message = "Username must be greater than 4 chars"),
        regex(
            path = "RE_USERNAME",
            message = "Username must be alphanumeric and/or dashes only"
        )
    )]
    pub username: String,

    #[validate(email(message = "invalid email"))]
    pub email: String,

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
) -> (Flash, impl IntoResponse) {
    match input.validate() {
        Ok(_) => {
            let user = UpsertUser {
                email: input.email,
                password: input.password,
                username: input.username,
            };

            if username_exists(&user.username, &state.pool).await {
                let flash = flash.error("Username is already taken");
                return (flash, Redirect::to("/signup"));
            }

            if email_exists(&user.email, &state.pool).await {
                let flash = flash.error("Email is already taken");
                return (flash, Redirect::to("/signup"));
            }

            User::create(state.pool, user).await;

            (flash, Redirect::to("/users"))
        }
        Err(errs) => {
            let errs = extract_errors(errs.errors());

            let f = errs.iter().fold(flash, |flash, (_, e)| flash.error(e));

            (f, Redirect::to("/signup"))
        }
    }
}

//sign-in user
// #[axum::debug_handler]
pub async fn signin_form(flash: Flash, flashes: IncomingFlashes) -> (Flash, impl IntoResponse) {
    #[derive(Template)]
    #[template(path = "users/signin.html")]
    struct Template<'a> {
        title: &'a str,
        flash: IncomingFlashes,
    }

    let templ = Template {
        title: "Sign In",
        flash: flashes,
    };

    (flash, Html(templ.render().unwrap()))
}

// post sign-up
#[derive(Deserialize, Serialize, Validate, Debug)]
pub struct SignInData {
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
}

// #[axum::debug_handler]
pub async fn signin(
    State(state): State<AppState>,
    session: Session,
    flash: Flash,
    Form(input): Form<SignInData>,
) -> (Flash, impl IntoResponse) {
    match input.validate() {
        Ok(_) => {
            let user = SignInData {
                password: input.password,
                username: input.username,
            };

            //get the user!!
            match User::signin(state.pool, user).await {
                Ok(user) => {
                    session.insert("user", user).unwrap();
                    let flash = flash.success("Welcome to our website");
                    return (flash, Redirect::to("/"));
                }
                Err(_) => {
                    let flash = flash.error("invalid credentials");
                    return (flash, Redirect::to("/signin"));
                }
            }
        }
        Err(errs) => {
            let errs = extract_errors(errs.errors());

            let f = errs.iter().fold(flash, |flash, (_, e)| flash.error(e));

            (f, Redirect::to("/signin"))
        }
    }
}

#[axum::debug_handler]
pub async fn signout(session: Session) -> impl IntoResponse {
    let _: Option<SignInData> = session.remove("user").unwrap();
    Redirect::to("/signin")
}
