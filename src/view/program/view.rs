use axum::http::Method;

#[derive(Clone)]
pub struct ProgramView {
    pub(crate) path: String,
    pub(crate) method: Method,
}
