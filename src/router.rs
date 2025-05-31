use tracing::debug;

pub trait RouterExt<S> {
    /// register_router registers the views in the ViewSet with the given axum router.
    fn register_router(&self, router: axum::Router<S>) -> Result<axum::Router<S>, crate::Error>;
}
