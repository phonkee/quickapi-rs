use axum::extract::rejection::PathRejection;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Improperly configured: {0}")]
    ImproperlyConfigured(String),

    #[error("Invalid method: {0}")]
    InvalidMethod(String),

    #[error("View error: {0}")]
    View(#[from] crate::view::Error),

    #[error("Lookup error: {0}")]
    Lookup(#[from] PathRejection),

    #[error("No query filter match")]
    NoQueryFilterMatch,
}
