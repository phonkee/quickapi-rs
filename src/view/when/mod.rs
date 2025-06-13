pub mod clause;
pub mod error;
mod views;
pub mod when;

pub use views::WhenViews;

/// When trait for defining conditions that must be met before executing a view
#[async_trait::async_trait]
pub trait When<S, T>: Send
where
    S: Clone + Send,
{
    /// Future type that will be returned when the condition is met
    // type Future: Future<Output = Result<(), error::Error>> + Send;

    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    async fn when(self, _parts: axum::http::request::Parts, _state: S) -> Result<(), error::Error>;
}
