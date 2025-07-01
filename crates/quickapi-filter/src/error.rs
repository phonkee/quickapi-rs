#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid query parameter: {0}")]
    InvalidQueryParameter(String),

    #[error("No match")]
    NoMatch,
}
