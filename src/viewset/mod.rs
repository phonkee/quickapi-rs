use crate::view::ViewTrait;
use crate::{Error, RouterExt};
use axum::Router;
use std::pin::Pin;
use tracing::debug;

/// ViewSet is a collection of views that can be registered with an axum router.
pub struct ViewSet<S> {
    path: String,
    views: Vec<
        Pin<
            Box<
                dyn ViewTrait<
                        S,
                        Future = Pin<
                            Box<
                                dyn Future<Output = Result<serde_json::Value, Error>>
                                    + Send
                                    + Sync
                                    + 'static,
                            >,
                        >,
                    >,
            >,
        >,
    >,
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
        }
    }

    /// add_view adds a view to the ViewSet.
    pub fn add_view(
        mut self,
        _view: impl ViewTrait<
            S,
            Future = Pin<
                Box<
                    dyn Future<Output = Result<serde_json::Value, Error>>
                        + Send
                        + Sync
                        + 'static,
                >,
            >,
        > + 'static,
    ) -> Self {
        self.views.push(Box::pin(_view));
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
        Ok(router.nest(&self.path.clone(), inner))
    }
}
