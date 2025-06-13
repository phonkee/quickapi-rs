use crate::RouterExt;
use axum::body::Body;
use axum::http::request::Parts;

/// View trait for defining a view (List, Get, Delete, Update, Create)
/// S is axum state type, which can be any type that implements Send + Sync.
/// TODO: use async_trait for the future type to allow for async operations.

#[async_trait::async_trait]
pub trait ViewTrait<S>: RouterExt<S> + Sync
where
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        parts: &mut Parts,
        state: S,
        body: Body,
    ) -> Result<crate::response::JsonResponse, crate::error::Error>;
}

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait ModelViewTrait<M, S>: ViewTrait<S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
}
