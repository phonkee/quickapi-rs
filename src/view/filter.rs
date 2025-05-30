use axum::extract::FromRequestParts;
use axum::extract::Request;
use axum::http::request::Parts;
use axum::response::Response;
use sea_orm::{EntityTrait, Select};
use std::pin::Pin;
use std::sync::Arc;

// Filter trait for any function taking Select<M> and extractors
pub trait Filter<S, M, Arg>: Send + Sync
where
    M: EntityTrait + Send + Sync + 'static,
{
    type Future: Future<Output = Result<Select<M>, ()>> + Send + 'static;

    fn call(self, req: Parts, state: S, s: Select<M>) -> Self::Future;
}

impl<S, M, F, R> Filter<S, M, f64> for F
where
    M: EntityTrait + Send + Sync + 'static,
    R: Future<Output = Result<Select<M>, ()>> + Send + 'static,
    F: Fn(Select<M>, Parts) -> R + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send + 'static>>;

    // call function
    fn call(self, parts: Parts, _state: S, s: Select<M>) -> Self::Future {
        Box::pin(async move { self(s, parts).await })
    }
}

impl<S, M, F, R> Filter<S, M, f32> for F
where
    M: EntityTrait + Send + Sync + 'static,
    R: Future<Output = Result<Select<M>, ()>> + Send + Sync + 'static,
    F: Fn(Select<M>, Parts, S) -> R + Send + Sync + 'static,
    S: Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send + 'static>>;

    // call function
    fn call(self, parts: Parts, _state: S, s: Select<M>) -> Self::Future {
        Box::pin(async move { self(s, parts, _state).await })
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
    fn call(self, _: Parts, _state: S, s: Select<M>) -> Self::Future {
        Box::pin(async move { self(s).await })
    }
}

impl<S, M, T1, T2> Filter<S, M, ((), ())> for (T1, T2)
where
    M: EntityTrait + Send + Sync + 'static,
    // R: Future<Output = Result<Select<M>, ()>> + Send + Sync + 'static,
    // F: Fn(Select<M>) -> R + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T1: Filter<S, M, ((), ())> + Send + Sync + 'static,
    T2: Filter<S, M, ((), ())> + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send + 'static>>;
    fn call(self, parts: Parts, _state: S, s: Select<M>) -> Self::Future {
        let state = _state.clone();
        Box::pin(async move {
            let Ok(s) = self.0.call(parts.clone(), state.clone(), s).await else {
                return Err(());
            };
            let Ok(s) = self.1.call(parts.clone(), state.clone(), s).await else {
                return Err(());
            };

            Ok(s)
        })
    }
}
