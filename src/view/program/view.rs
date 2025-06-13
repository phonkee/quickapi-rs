use crate::view::program::ProgramViewTrait;
use crate::view::when::WhenViews;
use axum::http::Method;
use std::sync::Arc;

/// new creates a new ProgramView with the specified path and defaults to the GET method.
pub fn new<S>(path: impl Into<String>, _what: impl ProgramViewTrait<S>) -> ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    new_with_method(path, Method::GET, _what)
}

/// new_with_method creates a new ProgramView with the specified path and method.
pub fn new_with_method<S>(
    path: impl Into<String>,
    method: Method,
    _what: impl ProgramViewTrait<S>,
) -> ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    ProgramView::<S> {
        path: path.into(),
        method,
        when: WhenViews::<S, Arc<dyn ProgramViewTrait<S> + Sync + Send + 'static>>::new(),
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) path: String,
    pub(crate) method: Method,
    pub(crate) when: WhenViews<S, Arc<dyn ProgramViewTrait<S> + Send + Sync + 'static>>,
}

/// ProgramViewTrait is a trait that defines the behavior of a program view.
impl<S> ProgramViewTrait<S> for ProgramView<S> where S: Clone + Send + Sync + 'static {}

