#![allow(unused_imports)]

use axum::extract::Request;
use axum::http::request::Parts;
use quickapi::view::list::ListView;
use sea_orm::Select;
use std::pin::Pin;
use tracing::info;
// pub async fn filter(
//     sel: sea_orm::Select<entity::User>,
//     _req: &mut axum::extract::Request,
// ) -> Result<sea_orm::Select<entity::User>, ()> {
//     // Box::pin(async move {
//     //     // Filtering logic here
//     Ok(sel)
//     // })
// }

pub async fn filter(_s: Select<entity::User>, _: Parts) -> Result<Select<entity::User>, ()> {
    Ok(_s)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // prepare tracing subscriber
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_thread_names(true)
        .with_line_number(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let router: axum::Router<()> = axum::Router::new().route(
        "/api/user",
        quickapi::view::get(
            // add list view for User entity
            ListView::<entity::User>::default()
                // add a condition to the view
                .when((), |view| {
                    // filter by something
                    view.filter(filter)
                    // view
                }),
        ),
    );

    // prepare listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4148").await?;

    // Log the address we're listening on
    info!("Listening on {}", listener.local_addr()?);

    // Serve the router
    Ok(axum::serve(listener, router).await?)
}
