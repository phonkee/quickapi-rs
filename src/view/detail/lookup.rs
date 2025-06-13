use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::http::request::Parts;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{EntityTrait, Select};
use std::marker::PhantomData;

/// Lookup for primary key or other unique identifier in the database.
pub trait Lookup<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn lookup(
        &self,
        parts: &mut Parts,
        _s: S,
        q: Select<M>,
    ) -> impl Future<Output = Result<Select<M>, crate::error::Error>>;
}

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
        let _: Path<String> = Path::from_request_parts(_parts, &_s).await?;

        Ok(q)
    }
}

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
