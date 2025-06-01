pub mod clause;
pub mod when;

use crate::view::View;
use crate::view::filter::Filter;
use crate::view::list::ListView;
use axum::body::Body;
use sea_orm::Select;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

pub use when::*;

pub trait When<S>: Send + Sync
where
    S: Clone + Send + Sync + 'static,
{
    type Future: Future<Output = Result<(), crate::view::error::Error>> + Send + Sync + 'static;

    fn when(self, req: &mut axum::extract::Request, _state: S) -> Self::Future;
}
