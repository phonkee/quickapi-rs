pub mod error;
mod filter;
pub mod list;
pub mod routing;
pub mod when;

pub use routing::get;

pub trait View {
    type Future: Future<Output = Result<serde_json::Value, crate::error::Error>>;

    /// list method to retrieve a list of items
    fn view<S>(&self, req: axum::extract::Request, state: S) -> Self::Future
    where
        S: Clone + Send + Sync + 'static;
}
