mod viewset;

use crate::view::ViewTrait;
use crate::{Error, RouterExt};
use axum::Router;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug_span;

/// new creates a new ViewSet with the given path.
pub(crate) fn new<S>(path: impl Into<String>) -> ViewSet<S> {
    ViewSet {
        path: path.into(),
        views: vec![],
        phantom_data: PhantomData,
    }
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
    /// add_view adds a view to the ViewSet.
    pub fn add_view(mut self, _view: impl ViewTrait<S> + Send + Sync + 'static) -> Self {
        self.views.push(Arc::new(_view));
        self
    }
}

/// Implementing RouterExt for ViewSet to register the views with the axum router.
impl<S> RouterExt<S> for ViewSet<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// register_router registers the views in the ViewSet with the given axum router.
    fn register_router_with_prefix(&self, router: Router<S>, _: &str) -> Result<Router<S>, Error> {
        let span = debug_span!("viewset", viewset_path = %self.path);
        let _enter = span.enter();

        // prepare new router
        let mut inner = Router::new();

        // register all views
        for view in &self.views {
            // no prefix for viewset, so we register with empty prefix
            inner = view.register_router_with_prefix(inner, &self.path.clone())?;
        }

        // return nested router
        // Ok(router.clone().nest(&self.path.clone(), inner))
        Ok(router.clone().merge(inner))
    }
}
