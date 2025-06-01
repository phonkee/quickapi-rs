pub mod clause;
pub mod error;
pub mod when;

use crate::view::filter::Filter;
use axum::body::Body;
use sea_orm::Select;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

pub use when::*;

/// When trait for defining conditions that must be met before executing a view
pub trait When<S, T>: Send + Sync
where
    S: Clone + Send + Sync + 'static,
{
    /// Future type that will be returned when the condition is met
    type Future: Future<Output = Result<(), error::Error>> + Send + Sync + 'static;

    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    fn when(self, _parts: &mut axum::http::request::Parts, _state: S) -> Self::Future;
}
