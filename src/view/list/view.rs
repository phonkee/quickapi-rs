#![allow(unused_mut)]

use crate::view::filter::Filter;
use crate::view::when::When;
use axum::response::{IntoResponse, Response};
use sea_orm::Select;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Default)]
pub struct ListView<M, S = ()>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    // a list of filters to apply to the query
    filters: Vec<
        Arc<
            Box<
                dyn Filter<
                        S,
                        M,
                        Future = Pin<
                            Box<dyn Future<Output = Result<Select<M>, ()>> + Send + 'static>,
                        >,
                    >,
            >,
        >,
    >,
    // when condition to apply logic
    when: Vec<
        Arc<
            Box<
                dyn When<
                    Future = Pin<
                        Box<
                            dyn Future<Output = Result<(), crate::view::error::Error>>
                                + Send
                                + 'static,
                        >,
                    >,
                >,
            >,
        >,
    >,
}

impl<M, S> Clone for ListView<M, S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        ListView {
            filters: self.filters.clone(),
            when: self.when.clone(),
        }
    }
}

/// ListView struct for handling list views of entities
impl<M, S> ListView<M, S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// when method to conditionally apply logic
    pub fn when<F>(mut self, _when: impl When, _f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        // Here you can implement logic to handle the `when` condition
        // For now, we just return self
        self
    }

    /// filter method to apply a filter condition
    pub fn filter<X>(mut self, _filter: impl Filter<S, M, X>) -> Self {
        self
    }
}

// Handler trait implementation for RequestHandler
impl<M, S> axum::handler::Handler<(), S> for ListView<M, S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    // Future type for the handler
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    // Call method to handle the request
    fn call(self, _req: axum::extract::Request, _state: S) -> Self::Future {
        Box::pin(
            async move { (axum::http::StatusCode::OK, "hello world".to_string()).into_response() },
        )
    }
}
