/*
 *  The MIT License (MIT)
 *
 *  Copyright (c) 2024-2025, Peter Vrba
 *
 *  Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the "Software"), to deal
 *  in the Software without restriction, including without limitation the rights
 *  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *  copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 *  all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 *  THE SOFTWARE.
 *
 */
mod serializers;

use axum::extract::Query;
use quickapi::filter_common::paginator::Paginator;
use quickapi::prelude::*;
use quickapi_lookup::PrimaryKey;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::Select;
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, info};

// primary_key_filter filters by primary key
pub async fn primary_key_filter(
    _query: Select<entity::User>,
    _x: axum::extract::OriginalUri,
    _y: axum::extract::OriginalUri,
) -> Result<Select<entity::User>, quickapi_filter::Error> {
    // get id query parameter
    Ok(_query)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct QuerySearch {
    query: Option<String>,
}

// filter_search_query filters the search query
pub async fn filter_search_query_username(
    query: Select<entity::User>,
    search: Query<QuerySearch>,
) -> Result<Select<entity::User>, quickapi_filter::Error> {
    // if query is present, filter by username
    Ok(if let Some(s) = search.0.query {
        query.filter(entity::user::Column::Username.contains(s))
    } else {
        query
    })
}


// MAX_DB_CONNECTION_TIMEOUT_SECONDS is the maximum time in seconds to wait for a database connection
const MAX_DB_CONNECTION_TIMEOUT_SECONDS: u64 = 5;

/// when_condition is a condition that will be checked before applying the view
pub async fn when_condition(_x: axum::extract::OriginalUri) -> Result<(), quickapi_when::Error> {
    Ok(())
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct QueryFormat {
    format: Option<String>,
}

/// when_condition is a condition that will be checked before applying the view
pub async fn when_condition_format(_x: Query<QueryFormat>) -> Result<(), quickapi_when::Error> {
    match &_x.format {
        Some(format) if format == "full" => Ok(()),
        _ => Err(quickapi_when::Error::NoMatch),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // prepare tracing subscriber
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_thread_names(true)
        // .with_line_number(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();


    // instantiate quickapi with database connection instance
    debug!("Connecting to database");

    // prepare database connection options
    let mut db_opts = sea_orm::ConnectOptions::new(
        "postgres://quickapi-example:quickapi-example@localhost:5432/quickapi-example",
    );
    let db_opts = db_opts.connect_timeout(Duration::from_secs(MAX_DB_CONNECTION_TIMEOUT_SECONDS));

    let api = quickapi::new::<()>(
        sea_orm::Database::connect(db_opts.clone())
            .await
            .expect("cannot connect to database"),
    );

    // prepare axum router instance so we can register views to it
    let router = axum::Router::new();

    // add list view for User entity
    let router = api
        .list::<entity::User>("/api/user")?
        .with_filter(Paginator::default())
        .with_filter(filter_search_query_username)
        .with_serializer::<serializers::UsernameOnly>()
        .wrap_result_key("users")
        .when(when_condition_format, |v| {
            // change serializer for this condition
            Ok(v.with_serializer::<serializers::SimpleUser>())
        })?.register_router(router)?;

    // add detail view for User entity
    let router = api
        .detail::<entity::User>("/api/user/{id}", PrimaryKey::Path("id".into()))?
        .with_serializer::<serializers::UsernameOnly>()
        .wrap_result_key("user")
        .when(when_condition_format, |v| {
            Ok(v.with_serializer::<serializers::SimpleUser>())
        })?.register_router(router)?;

    // Create View example (Still in development)
    let router = api
        .create::<entity::User>("/api/user")?
        .with_serializer::<serializers::CreateUser>()
        .with_before_save(async move |m: entity::UserModel| {
            // do something with model before saving
            debug!("Before save: {:?}", m);
            Ok(m)
        })
        .register_router(router)?;


    // // add multiple prefixed views as a tuple and use single register_router call
    // let router = (
    //     api.prefix("/api/internal/order/")
    //          api.delete::<entity::Order>("/secret/{pk}", "id")?
    //              .with_lookup("pk"),
    //     ),
    //     api.detail::<entity::Order>("/secret/{pk}", "pk")?,
    //     (
    //         // if you exceed the maximum number of views, you can use tuple to group them
    //         api.list::<entity::Order>("/secret/")?,
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
