use askama::Template;
use axum::response::{Html, IntoResponse};

//---- Index  ---///
pub async fn index() -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "index.html")]
    struct MainTemplate<'a> {
        title: &'a str,
    }

    let hello = MainTemplate { title: "Main Page" };
    Html(hello.render().unwrap())
}
