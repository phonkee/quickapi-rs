#![allow(unused_imports)]

mod serializers;

use axum::extract::{Path, Request};
use axum::http::Method;
use axum::http::request::Parts;
use quickapi::router::RouterExt;
use quickapi::view;
use quickapi::when::when::*;
use sea_orm::{EntityTrait, Iden, Select};
use std::marker::PhantomData;
use std::pin::Pin;
use tracing::info;

/// Filter user
pub async fn filter_user(_s: Select<entity::User>, _: Parts) -> Result<Select<entity::User>, ()> {
    Ok(_s)
}

/// when_condition is a condition that will be checked before applying the view
#[allow(unused_variables)]
pub async fn when_condition(
    _parts: Parts,
    _state: (),
    Path((user_id, team_id)): Path<(String, String)>,
) -> Result<(), quickapi::Error> {
    Ok(())
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

    // connect sea-orm to the database
    let api = quickapi::new::<()>(
        sea_orm::Database::connect(
            "postgres://quickapi-example:quickapi-example@localhost:5432/quickapi-example",
        )
        .await?,
    );

    println!("DB: {:?}", api);

    // router instance
    let router: axum::Router<()> = axum::Router::new();

    // try new api
    let router = api
        .view()
        .detail()
        .new::<entity::User>("/hello")?
        .with_lookup("id")
        .when(when_condition, |mut v| {
            Ok(v.with_serializer::<serializers::SimpleUser>())
        })?
        .register_router(router)?;

    // add list view for User entity
    let router = api
        .view()
        .list()
        .new::<entity::User>("/api/user")?
        .when(when_condition, |v| {
            // filter by something
            Ok(
                v.with_serializer::<serializers::SimpleUser>(), // .with_filter(|_parts, _state, query| Box::pin(async move { Ok(query) }))
            )
        })?
        .register_router(router)?;

    // add viewset for Order entity
    let router = api.viewset("/api/order")
        // .add_view(view::detail::new::<entity::Order, ()>("/{pk}")?.with_lookup("pk"))
        .add_view(view::delete::new::<entity::Order, ()>("/{pk}")?.with_lookup("pk"))
        .register_router(router)?;

    // // add views from tuple
    // let router = (
    //     view::list::new::<entity::User, ()>("/api/internal/user"),
    //     // view::detail::new::<entity::User, ()>("/api/internal/user/{id}")?,
    //     view::delete::new::<entity::User, ()>("/api/internal/user/{id}")?,
    //     (
    //         view::list::new::<entity::User, ()>("/api/external/user"),
    //         // view::detail::new::<entity::User, ()>("/api/external/user/{id}")?,
    //     ),
    // )
    //     .register_router(router)?;

    // prepare listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4148").await?;

    // Log the address we're listening on
    info!("Listening on {}", listener.local_addr()?);

    // Serve the router
    Ok(axum::serve(listener, router).await?)
}
