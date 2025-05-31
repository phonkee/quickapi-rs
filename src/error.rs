use axum::routing::MethodFilter;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid method: {0}")]
    InvalidMethod(String),

    #[error("View error: {0}")]
    View(#[from] crate::view::Error),
}
