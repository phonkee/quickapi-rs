#![allow(unused_imports)]

mod serializers;

use axum::extract::Request;
use axum::http::Method;
use axum::http::request::Parts;
use quickapi::router::RouterExt;
use quickapi::view;
use quickapi::view::when::when::*;
use sea_orm::{EntityTrait, Iden, Select};
use std::marker::PhantomData;
use std::pin::Pin;
use tracing::info;

/// Filter user
pub async fn filter_user(_s: Select<entity::User>, _: Parts) -> Result<Select<entity::User>, ()> {
    Ok(_s)
}

/// when_condition is a condition that will be checked before applying the view
pub async fn when_condition(_parts: Parts, _state: ()) -> Result<(), view::when::error::Error> {
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

    // router instance
    let router: axum::Router<()> = axum::Router::new();

    // // add list view for User entity
    // let router = view::list::new::<entity::User, ()>("/api/user")
    //     .with_method(Method::GET)
    //     // add a condition to the view
    //     .when(
    //         when_condition,
    //         |view: view::list::ListView<entity::User, <entity::User as EntityTrait>::Model, ()>| {
    //             // filter by something
    //             // view.filter(filter).with_serializer::<UserIdOnly>()
    //             Ok(view)
    //         },
    //     )
    //     .register_router(router)?;

    let router =
        view::detail::new::<entity::User, ()>("/api/user/{id}", "id").register_router(router)?;

    // // add list view for User entity
    // let router =
    //     view::list::ListView::<entity::User, <entity::User as EntityTrait>::Model, ()>::new(
    //         "/api/user",
    //         Method::GET,
    //     )
    //     // add a condition to the view
    //     .when(
    //         when_condition,
    //         |view: view::list::ListView<entity::User, <entity::User as EntityTrait>::Model, ()>| {
    //             // filter by something
    //             // view.filter(filter).with_serializer::<UserIdOnly>()
    //             Ok(view)
    //         },
    //     )
    //     .register_router(router)?;

    // // add viewset for User entity
    // let router = quickapi::ViewSet::new("/api/viewset/user")
    //     .add_view(view::list::ListView::<
    //         entity::User,
    //         <entity::User as EntityTrait>::Model,
    //         (),
    //     >::new("/", Method::GET))
    //     .add_view(view::detail::DetailView::<
    //         entity::User,
    //         <entity::User as EntityTrait>::Model,
    //         (),
    //     >::new("/{id}", Method::GET, "id".to_string()))
    //     .register_router(router)?;

    // prepare listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4148").await?;

    // Log the address we're listening on
    info!("Listening on {}", listener.local_addr()?);

    // Serve the router
    Ok(axum::serve(listener, router).await?)
}
