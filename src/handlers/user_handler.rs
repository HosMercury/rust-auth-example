use std::collections::HashMap;
use std::env;

use crate::models::user_model::{GetUser, UpsertUser};
use crate::validation::extract_errors;
use crate::{filters, validation};
use crate::{models::user_model::User, AppState};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Form, Json,
};
use axum_flash::{Flash, IncomingFlashes};
use oauth2::{
    basic::BasicClient,
    basic::{BasicErrorResponseType, BasicTokenType},
    reqwest::async_http_client,
    url::Url,
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RedirectUrl, RevocationErrorResponseType, Scope, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse, TokenUrl,
};
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
pub async fn signup_form(
    session: Session,
    flash: Flash,
    flashes: IncomingFlashes,
) -> (Flash, impl IntoResponse) {
    let authorize_url = oauth_url();

    session
        .insert("authorize_url", authorize_url.clone())
        .unwrap();

    #[derive(Template)]
    #[template(path = "users/signup.html")]
    struct Template<'a> {
        title: &'a str,
        flash: IncomingFlashes,
        authorize_url: Url,
    }

    let templ = Template {
        title: "Sign Up",
        flash: flashes,
        authorize_url,
    };

    (flash, Html(templ.render().unwrap()))
}

////////////////////////////////////////////////////////////////////////
////////////////////////////// AUTH ////////////////////////////////////
/// ////////////////////////////////////////////////////////////////////

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
pub async fn signin_form(
    session: Session,
    flash: Flash,
    flashes: IncomingFlashes,
) -> (Flash, impl IntoResponse) {
    let authorize_url = oauth_url();

    session.insert("authorize_url", &authorize_url).unwrap();

    #[derive(Template)]
    #[template(path = "users/signin.html")]
    struct Template<'a> {
        title: &'a str,
        flash: IncomingFlashes,
        authorize_url: Url,
    }

    let templ = Template {
        title: "Sign In",
        flash: flashes,
        authorize_url,
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
                    (flash, Redirect::to("/"))
                }
                Err(_) => {
                    let flash = flash.error("invalid credentials");
                    (flash, Redirect::to("/signin"))
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
    session.remove::<SignInData>("user").unwrap();
    Redirect::to("/signin")
}

///////////////////////////////////////////////////////////////////////////////
/////////////////////////// Oauth /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////

fn oauth_client() -> Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    BasicTokenType,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
> {
    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_SECRET").expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8899/g/callback".to_string())
            .expect("Invalid redirect URL"),
    );

    client
}

fn oauth_url() -> Url {
    let client = oauth_client();
    let scope_profile: &str = "https://www.googleapis.com/auth/userinfo.profile";
    let scope_email: &str = "https://www.googleapis.com/auth/userinfo.email";

    let (authorize_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(scope_profile.to_string()))
        .add_scope(Scope::new(scope_email.to_string()))
        .url();

    authorize_url
}

#[derive(Deserialize)]
pub struct RedirectQuery {
    pub code: String,
}

pub async fn google_oauth_callback(Query(q): Query<RedirectQuery>) {
    let RedirectQuery { code } = q;
    let code = AuthorizationCode::new(code);

    let client = oauth_client();

    let token_response = client
        .exchange_code(code)
        .request_async(async_http_client)
        .await
        .unwrap();

    println!(
        "Google returned the following token:\n{:?}\n",
        token_response.access_token().secret()
    );

    // link to get user data
    // https://www.googleapis.com/oauth2/v3/userinfo

    let user_info_url = "https://www.googleapis.com/oauth2/v3/userinfo";
    let token = token_response.access_token().secret();
    let client = reqwest::Client::new();

    let request = client
        .get(user_info_url)
        .header("Authorization", format!("Bearer {}", token));

    let response = request.send().await.unwrap();

    #[derive(Debug, Serialize, Deserialize)]
    struct ResponseData {
        sub: u64,
        name: String,
        picture: Option<String>,
        email: String,
    }

    // println!("resp json {}", response.text().await.unwrap());

    // if the request was successful (status code 2xx)
    if response.status().is_success() {
        let response_data: ResponseData = response.json().await.unwrap();
        println!("Response: {:#?}", response_data);
    } else {
        println!(
            "Error: {} - {}",
            response.status(),
            response.text().await.unwrap()
        );
    }
}
