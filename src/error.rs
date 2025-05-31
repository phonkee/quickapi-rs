use axum::routing::MethodFilter;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Match error: {0}")]
    Match(String),

    #[error("Method: {0} not")]
    MethodFilter(String),
}
