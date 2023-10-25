use axum::{
    routing::{get, post},
    Router,
};
use controllers::users_controller;

use crate::controllers;

pub fn web() -> Router {
    return Router::new().route("/users/create", post(users_controller::create_user));
}
