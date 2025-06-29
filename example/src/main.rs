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
use quickapi::RouterExt;
use quickapi::ViewWrapResultTrait;
use quickapi::filter_common::paginator::Paginator;
use quickapi_lookup::PrimaryKeyLookup;
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
    _query: Select<entity::User>,
    _search: Query<QuerySearch>,
) -> Result<Select<entity::User>, quickapi_filter::Error> {
    // if query is present, filter by username
    let _query = match _search.0 {
        QuerySearch { query: Some(q) } => _query.filter(entity::user::Column::Username.contains(q)),
        _ => _query,
    };

    Ok(_query)
}

// MAX_DB_CONNECTION_TIMEOUT_SECONDS is the maximum time in seconds to wait for a database connection
const MAX_DB_CONNECTION_TIMEOUT_SECONDS: u64 = 5;

/// when_condition is a condition that will be checked before applying the view
pub async fn when_condition(_x: axum::extract::OriginalUri) -> Result<(), quickapi_when::Error> {
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

    // prepare database connection options
    let mut db_opts = sea_orm::ConnectOptions::new(
        "postgres://quickapi-example:quickapi-example@localhost:5432/quickapi-example",
    );
    let db_opts = db_opts.connect_timeout(Duration::from_secs(MAX_DB_CONNECTION_TIMEOUT_SECONDS));

    // instantiate quickapi with database connection instance
    debug!("Connecting to database");
    let api = quickapi::new::<()>(
        sea_orm::Database::connect(db_opts.clone())
            .await
            .expect("cannot connect to database"),
    );

    // prepare axum router instance so we can register views(viewsets) to it
    let router = axum::Router::new();

    // // try new api
    // let router = api
    //     .detail::<entity::User>("/hello/world/{id}", "id")?
    //     .when(when_condition, |v| {
    //         Ok(v.with_serializer::<serializers::SimpleUser>())
    //     })?
    //     .register_router(router)?;

    // add list view for User entity
    let router = api
        .list::<entity::User>("/api/user")?
        .with_filter(Paginator::default())
        .with_filter(primary_key_filter)
        .with_filter(filter_search_query_username)
        .with_serializer::<serializers::UsernameOnly>()
        .wrap_result_key("users")
        // .when(when_condition, |v| {
        //     // change serializer for this condition
        //     Ok(v.with_serializer::<serializers::SimpleUser>())
        // })?
        .register_router(router)?;

    // add viewset for Order entity
    let router =
        api.prefix("/api/order")
            .with_filter(api.delete::<entity::Order>("/{pk}")?.with_lookup("pk"))
            .with_filter(api.detail::<entity::Order>(
                "some/order/{pk}",
                PrimaryKeyLookup::Path("pk".to_owned()),
            )?)
            .register_router(router)?;

    // add multiple prefixes and from tuple of views
    let router = (
        api.prefix("/api/internal/order/").with_filter(
            api.prefix("secret/")
                .with_filter(api.detail::<entity::Order>("some/order/{pk}", "pk")?),
        ),
        api.delete::<entity::Order>("/secret/{pk}")?
            .with_lookup("pk"),
        api.detail::<entity::Order>("/secret/{pk}", "pk")?,
        (
            // if you exceed the maximum number of views, you can use tuple to group them
            api.list::<entity::Order>("/secret/")?,
        ),
    )
        .register_router(router)?;

    // TODO: create view
    let router = api
        .create::<entity::User>("/api/user")?
        .with_serializer::<serializers::SimpleUser>()
        .with_before_save(async move |m: entity::UserModel| {
            // do something with model before saving
            debug!("Before save: {:?}", m);
            Ok(m)
        })
        .register_router(router)?;

    // prepare listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4148").await?;

    // Log the address we're listening on
    info!("Listening on {}", listener.local_addr()?);

    // Serve the router
    Ok(axum::serve(listener, router).await?)
}
