use axum::extract::Request;
use axum::response::Response;
use sea_orm::{EntityTrait, Select};
use std::pin::Pin;

// Filter trait for any function taking Select<M> and extractors
pub trait Filter<S, M, Arg>: Send + Sync + 'static
where
    M: EntityTrait + Send + Sync + 'static,
{
    type Future: Future<Output = Result<Select<M>, ()>> + Send + 'static;

    fn call(self, req: Request, state: S, s: Select<M>) -> Self::Future;
}

impl<S, M, F, R> Filter<S, M, f64> for F
where
    M: EntityTrait + Send + Sync + 'static,
    R: Future<Output = Result<Select<M>, ()>> + Send + 'static,
    F: Fn(Request, Select<M>) -> R + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send + 'static>>;

    // call function
    fn call(self, _req: Request, _state: S, s: Select<M>) -> Self::Future {
        Box::pin(async move { self(_req, s).await })
    }
}

impl<S, M, F, R> Filter<S, M, f32> for F
where
    M: EntityTrait + Send + Sync + 'static,
    R: Future<Output = Result<Select<M>, ()>> + Send + Sync + 'static,
    F: Fn(Request, S, Select<M>) -> R + Send + Sync + 'static,
    S: Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send + 'static>>;

    // call function
    fn call(self, _req: Request, _state: S, s: Select<M>) -> Self::Future {
        Box::pin(async move { self(_req, _state, s).await })
    }
}

impl<S, M, F, R> Filter<S, M, u8> for F
where
    M: EntityTrait + Send + Sync + 'static,
    R: Future<Output = Result<Select<M>, ()>> + Send + Sync + 'static,
    F: Fn(Select<M>) -> R + Send + Sync + 'static,
    S: Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send + 'static>>;

    // call function
    fn call(self, _req: Request, _state: S, s: Select<M>) -> Self::Future {
        Box::pin(async move { self(s).await })
    }
}
    