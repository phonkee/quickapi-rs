pub mod error;
pub mod filter;
pub mod list;
pub mod when;

use axum::http::request::Parts;
use axum::routing::{MethodFilter, on};

pub use error::Error;

pub trait View<S> {
    type Future: Future<Output = Result<serde_json::Value, crate::error::Error>>;

    /// list method to retrieve a list of items
    fn view(&self, parts: &mut Parts, state: S) -> Self::Future;

    /// register_router method to register the view with an axum router
    fn register_router(
        &self,
        router: axum::Router<S>,
    ) -> Result<axum::Router<S>, crate::error::Error>;
}
