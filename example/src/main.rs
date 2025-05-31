#![allow(unused_imports)]

use axum::extract::Request;
use axum::http::Method;
use axum::http::request::Parts;
use quickapi::view::list::ListView;
use sea_orm::{EntityTrait, Select};
use std::pin::Pin;
use tracing::info;

pub async fn filter(_s: Select<entity::User>, _: Parts) -> Result<Select<entity::User>, ()> {
    Ok(_s)
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct SimpleUser {
    pub id: i32,
    pub username: String,
}

impl From<entity::UserModel> for SimpleUser {
    fn from(user: entity::UserModel) -> Self {
        SimpleUser {
            id: user.id,
            username: user.username,
        }
    }
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct UserIdOnly {
    pub id: i32,
}

impl From<entity::UserModel> for UserIdOnly {
    fn from(user: entity::UserModel) -> Self {
        UserIdOnly { id: user.id }
    }
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

    let router: axum::Router<()> = axum::Router::new();

    // // add list view for User entity
    // let router = ListView::<entity::User, (), <entity::User as EntityTrait>::Model>::new(
    //     "/api/user",
    //     Method::GET,
    // )
    // // add a condition to the view
    // .when(
    //     (),
    //     |view: ListView<entity::User, (), <entity::User as EntityTrait>::Model>| {
    //         // filter by something
    //         // view.filter(filter).with_serializer::<UserIdOnly>()
    //         Ok(view)
    //     },
    // )
    // .register_axum(router)?;

    let router = quickapi::ViewSet::new("/api/viewset/user")
        .add_view(ListView::<
            entity::User,
            (),
            <entity::User as EntityTrait>::Model,
        >::new("/", Method::GET))
        .register_axum(router)?;

    // prepare listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4148").await?;

    // Log the address we're listening on
    info!("Listening on {}", listener.local_addr()?);

    // Serve the router
    Ok(axum::serve(listener, router).await?)
}
