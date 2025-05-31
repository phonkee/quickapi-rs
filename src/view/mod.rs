pub mod error;
pub mod list;
pub mod when;
pub mod filter;
mod detail;

use axum::http::request::Parts;
use crate::router::RouterExt;
pub use error::Error;

/// View trait for defining a view (List, Get, Delete, Update, Create)
/// TODO: This trait is still in development and may change in the future.
/// S is axum state type, which can be any type that implements Send + Sync.
pub trait View<S>: RouterExt<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Future type for the view method
    type Future: Future<Output = Result<serde_json::Value, crate::error::Error>>;

    /// list method to retrieve a list of items
    fn view(&self, parts: &mut Parts, state: S) -> Self::Future;
}
