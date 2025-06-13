use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::http::request::Parts;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{EntityTrait, Select};
use std::marker::PhantomData;
use tracing::debug;

/// Lookup for primary key or other unique identifier in the database.
#[async_trait::async_trait]
pub trait Lookup<M, S>: Send + Sync
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    // lookup method filters the query based on the provided parts and state.
    async fn lookup(
        &self,
        parts: &mut Parts,
        _s: S,
        q: Select<M>,
    ) -> Result<Select<M>, crate::error::Error>;
}

#[async_trait::async_trait]
impl<M, S> Lookup<M, S> for String
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn lookup(
        &self,
        _parts: &mut Parts,
        _s: S,
        q: Select<M>,
    ) -> Result<Select<M>, crate::error::Error> {
        let _id: Path<String> = Path::from_request_parts(_parts, &_s).await?;
        debug!("Lookup by field: {:?}", _id.0);

        Ok(q)
    }
}

#[async_trait::async_trait]
impl<M, S> Lookup<M, S> for &str
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn lookup(
        &self,
        _parts: &mut Parts,
        _s: S,
        q: Select<M>,
    ) -> Result<Select<M>, crate::error::Error> {
        let _: Path<String> = Path::from_request_parts(_parts, &_s).await?;
        Ok(q)
    }
}
