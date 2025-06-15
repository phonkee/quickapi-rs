#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No match")]
    NoMatch,
}
