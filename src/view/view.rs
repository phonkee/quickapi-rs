use crate::RouterExt;
use axum::http::request::Parts;

/// View trait for defining a view (List, Get, Delete, Update, Create)
/// TODO: This trait is still in development and may change in the future.
/// S is axum state type, which can be any type that implements Send + Sync.
pub trait ViewTrait<S>: RouterExt<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Future type for the view method
    type Future: Future<Output = Result<serde_json::Value, crate::error::Error>>;

    /// handle_view for view
    fn handle_view(&self, parts: &mut Parts, state: S) -> Self::Future;
}
