#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No match")]
    NoMatch,
}
