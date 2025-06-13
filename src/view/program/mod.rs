mod view;

use axum::http::Method;
pub use view::ProgramView;

/// new creates a new ProgramView with the specified path and defaults to the GET method.
pub fn new(path: impl Into<String>) -> ProgramView {
    ProgramView {
        path: path.into(),
        method: Method::GET,
    }
}

