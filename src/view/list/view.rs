#![allow(unused_mut)]

use crate::view::when::When;
use axum::response::{IntoResponse, Response};
use std::marker::PhantomData;
use std::pin::Pin;

#[derive(Clone, Default)]
pub struct View<M, S = ()> {
    _phantom: PhantomData<M>,
    _phantom2: PhantomData<S>,
}

impl<M, S> View<M, S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    pub fn when<F>(mut self, _when: impl When, _f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        // Here you can implement logic to handle the `when` condition
        // For now, we just return self
        self
    }
}

// Handler trait implementation for RequestHandler
impl<M, S> axum::handler::Handler<(), S> for View<M, S>
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
