pub mod error;
mod filter;
pub mod list;
pub mod routing;
pub mod when;

use axum::http::request::Parts;
use axum::routing::{MethodFilter, on};
pub use routing::get;

pub trait View<S> {
    type Future: Future<Output = Result<serde_json::Value, crate::error::Error>>;

    /// list method to retrieve a list of items
    fn view(&self, parts: &mut Parts, state: S) -> Self::Future;

    /// register_axum method to register the view with an axum router
    fn register_axum(
        &self,
        router: axum::Router<S>,
    ) -> Result<axum::Router<S>, crate::error::Error>;
}
