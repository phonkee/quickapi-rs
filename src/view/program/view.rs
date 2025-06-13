use crate::view::program::ProgramViewTrait;
use crate::view::when::WhenViews;
use axum::http::Method;

/// new creates a new ProgramView with the specified path and defaults to the GET method.
pub fn new<S>(path: impl Into<String>) -> ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    new_with_method(path, Method::GET)
}

/// new_with_method creates a new ProgramView with the specified path and method.
pub fn new_with_method<S>(path: impl Into<String>, method: Method, _what: impl Into<ProgramViewTrait<S>>) -> ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    ProgramView::<S> {
        path: path.into(),
        method,
        when: WhenViews::<S, dyn ProgramViewTrait>::new(),
    }
}

#[derive(Clone)]
pub struct ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) path: String,
    pub(crate) method: Method,
    pub(crate) when: WhenViews<S, dyn ProgramViewTrait<S>>,
}

impl<S> ProgramViewTrait<S> for ProgramView<S> where S: Clone + Send + Sync + 'static {}
