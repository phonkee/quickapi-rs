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
pub trait When<'a, S, T>: Send
where
    S: Clone + Send,
{
    /// Future type that will be returned when the condition is met
    type Future: Future<Output = Result<(), error::Error>> + Send;

    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    fn when(self, _parts: &'a mut axum::http::request::Parts, _state: S) -> Self::Future;
}
