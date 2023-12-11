use askama::Template;
use axum::response::{Html, IntoResponse};
use axum_flash::{Flash, IncomingFlashes};

// #[axum::debug_handler]
pub async fn index(flash: Flash, flashes: IncomingFlashes) -> (Flash, impl IntoResponse) {
    #[derive(Template)]
    #[template(path = "index.html")]
    struct MainTemplate<'a> {
        title: &'a str,
        flash: IncomingFlashes,
    }

    let base = MainTemplate {
        title: "Main Page",
        flash: flashes,
    };

    (flash, Html(base.render().unwrap()))
}
