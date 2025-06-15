mod views;
pub mod when;

pub use views::{WhenView, WhenViews};

/// When trait for defining conditions that must be met before executing a view
#[async_trait::async_trait]
pub trait When<S, T>: Send
where
    S: Clone + Send,
{
    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    async fn when(self, _parts: axum::http::request::Parts, _state: S) -> Result<(), crate::Error>;
}

/// CloneNoWhen trait for cloning objects without the Whens
pub trait CloneNoWhen {
    /// Clone the object without the When trait
    fn clone_without_when(&self) -> Self;
}
