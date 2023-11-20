use askama::Template;
use axum::response::{Html, IntoResponse};

#[axum::debug_handler]
pub async fn index() -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "base.j2")]
    struct MainTemplate<'a> {
        title: &'a str,
    }

    let base = MainTemplate { title: "Main Page" };
    Html(base.render().unwrap())
}
