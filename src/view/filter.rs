use axum::extract::FromRequest;
use axum::http::Request;
use axum::response::Response;
use sea_orm::{EntityTrait, Select};
use std::future::Future;
use std::pin::Pin;

pub trait Filter<S, M>: Clone + Send + Sync + Sized + 'static
where
    M: EntityTrait + Send + Sync + 'static,
{
    /// The type of future calling this handler returns.
    type Future: Future<Output = Result<Select<M>, ()>> + Send + 'static;

    // /// Call the handler with the given request.
    fn call(self, req: axum::extract::Request, s: Select<M>) -> Self::Future;
}

impl<S, M, F, R> Filter<S, M> for F
where
    S: Clone + Send + Sync + 'static,
    M: EntityTrait + Send + Sync + 'static,
    R: Future<Output = Result<Select<M>, ()>> + Send + 'static,
    F: Fn(Select<M>, axum::extract::Request) -> R + Clone + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send>>;

    fn call(self, req: axum::extract::Request, s: Select<M>) -> Self::Future {
        Box::pin(self(s, req))
    }
}
