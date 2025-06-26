/*
 * The MIT License (MIT)
 *
 * Copyright (c) 2024-2025, Peter Vrba
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */
mod serializers;

use axum::extract::Path;
use axum::http::request::Parts;
use quickapi::RouterExt;
use quickapi::filter_common::paginator::Paginator;
use sea_orm::Select;
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

// MAX_DB_CONNECTION_TIMEOUT_SECONDS is the maximum time in seconds to wait for a database connection
const MAX_DB_CONNECTION_TIMEOUT_SECONDS: u64 = 5;

/// when_condition is a condition that will be checked before applying the view
pub async fn when_condition(
    _parts: Parts,
    _state: (),
    Path(_id): Path<String>,
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

    // try new api
    let router = api
        .detail()
        .new::<entity::User>("/hello/world/{id}")?
        .with_lookup("id")
        // .when(when_condition, |mut v| {
        //     Ok(v.with_serializer::<serializers::SimpleUser>())
        // })?
        .register_router(router)?;

    // add list view for User entity
    let router = api
        .list()
        .new::<entity::User>("/api/user")?
        .with_filter(Paginator::default())
        .with_filter(primary_key_filter)
        // .when(when_condition, |v| {
        //     // filter by something
        //     Ok(
        //         v.with_serializer::<serializers::SimpleUser>(), // .with_filter(|_parts, _state, query| Box::pin(async move { Ok(query) }))
        //     )
        // })?
        .register_router(router)?;

    // add viewset for Order entity
    let router = api
        .prefix("/api/order")
        .add_view(
            api.delete()
                .new::<entity::Order>("/{pk}")?
                .with_lookup("pk"),
        )
        .register_router(router)?;

    // add views from tuple
    let router = (
        api.delete()
            .new::<entity::Order>("/secret/{pk}")?
            .with_lookup("pk"),
        api.detail()
            .new::<entity::Order>("/secret/{pk}")?
            .with_lookup("pk"),
        (api.list().new::<entity::Order>("/secret/")?,),
    )
        .register_router(router)?;

    // prepare listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4148").await?;

    // Log the address we're listening on
    info!("Listening on {}", listener.local_addr()?);

    // Serve the router
    Ok(axum::serve(listener, router).await?)
}
