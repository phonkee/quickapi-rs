#![allow(unused_imports)]

use quickapi::view::list::View as ListView;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router: axum::Router<()> = axum::Router::new().route(
        "/api/user",
        quickapi::view::get(ListView::<entity::User>::default()),
    );

    // prepare listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4148").await?;

    // Serve the router
    Ok(axum::serve(listener, router).await?)
}
