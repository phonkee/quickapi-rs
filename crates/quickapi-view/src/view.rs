/// ViewTrait defines the behavior of a view in the application.
#[async_trait::async_trait]
pub trait ViewTrait<S>: crate::RouterExt<S> + Sync
where
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        parts: &mut axum::http::request::Parts,
        state: S,
        body: axum::body::Body,
    ) -> Result<quickapi_http::response::Response, crate::Error>;
}

#[async_trait::async_trait]
impl<S> ViewTrait<S> for ()
where
    S: Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        _parts: &mut axum::http::request::Parts,
        _state: S,
        _body: axum::body::Body,
    ) -> Result<quickapi_http::response::Response, crate::Error> {
        Ok(quickapi_http::response::Response::default())
    }
}

/// ModelViewTrait defines the behavior of a model view in the application.
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait ModelViewTrait<M, S>: ViewTrait<S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        parts: &mut axum::http::request::Parts,
        state: S,
        body: axum::body::Body,
    ) -> Result<quickapi_http::response::Response, crate::error::Error>;
}
