mod controllers;
mod models;
mod routes;

#[tokio::main]
pub async fn run() {
    use std::net::SocketAddr;

    tracing_subscriber::fmt::init();

    let app = routes::web();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
