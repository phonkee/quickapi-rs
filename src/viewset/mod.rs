use crate::view::ViewTrait;
use crate::{Error, RouterExt};
use axum::Router;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

/// new creates a new ViewSet with the given path.
pub fn new<S>(path: impl Into<String>) -> ViewSet<S>
where
    S: Clone + Send + Sync + 'static,
{
    ViewSet::new(path)
}

/// ViewSet is a collection of views that can be registered with an axum router.
pub struct ViewSet<S> {
    path: String,
    views: Vec<Arc<dyn ViewTrait<S>>>,
    phantom_data: PhantomData<S>,
}

#[allow(unused_mut)]
impl<S> ViewSet<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new ViewSet with the given path.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            views: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    /// add_view adds a view to the ViewSet.
    #[allow(unused_mut)]
    pub fn add_view(mut self, _view: impl ViewTrait<S> + Send + Sync + 'static) -> Self {
        // TODO: add view to the ViewSet
        self.views.push(Arc::new(_view));
        self
    }
}

impl<S> RouterExt<S> for ViewSet<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// register_router registers the views in the ViewSet with the given axum router.
    fn register_router_with_prefix(&self, router: Router<S>, _: &str) -> Result<Router<S>, Error> {
        debug!("viewset: {} : views: {}", self.path, self.views.len());

        // prepare new router
        let mut inner = Router::new();

        // register all views
        for view in &self.views {
            inner = view.register_router_with_prefix(inner, &self.path.clone())?;
        }

        // return nested router
        Ok(router.clone().nest(&self.path.clone(), inner))
    }
}
