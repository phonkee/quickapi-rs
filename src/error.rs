use axum::extract::rejection::PathRejection;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Improperly configured: {0}")]
    ImproperlyConfigured(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Invalid method: {0}")]
    InvalidMethod(String),

    #[error("View error: {0}")]
    View(#[from] crate::view::Error),

    #[error("Lookup error: {0}")]
    Lookup(#[from] PathRejection),

    #[error("No query filter match")]
    NoQueryFilterMatch,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("No match when")]
    NoMatchWhen,
}
