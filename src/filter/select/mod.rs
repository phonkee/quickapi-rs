use crate::Error;
use axum::http::request::Parts;
use sea_orm::Select;

pub mod model;

#[async_trait::async_trait]
pub trait Filter<M, S, T>: Clone + Sync + Send + 'static
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn filter_select(
        &self,
        parts: &mut Parts,
        state: S,
        query: Select<M>,
    ) -> Result<Select<M>, Error>;
}
