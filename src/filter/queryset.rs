use crate::Error;
use async_trait::async_trait;
use axum::http::request::Parts;
use sea_orm::Select;

#[async_trait::async_trait]
pub trait SelectFilter<M, S>: Sync
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn filter_queryset(
        &self,
        parts: &mut axum::http::request::Parts,
        state: S,
        query: sea_orm::Select<M>,
    ) -> Result<sea_orm::Select<M>, crate::error::Error>;
}

#[async_trait::async_trait]
impl<M, S, H> SelectFilter<M, S> for H
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    H: axum::handler::Handler<(), S>,
    S: Sync + Send + Clone + 'static,
{
    async fn filter_queryset(
        &self,
        _parts: &mut axum::http::request::Parts,
        _state: S,
        query: sea_orm::Select<M>,
    ) -> Result<sea_orm::Select<M>, crate::error::Error> {
        Ok(query)
    }
}
