use crate::Error;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::http::request::Parts;
use sea_orm::Iterable;
use sea_orm::{ColumnTrait, EntityTrait, PrimaryKeyToColumn, QueryFilter, Select};
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

/// String implementation of Lookup trait. It does lookup by a primary key.
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

/// &str implementation of Lookup trait. It does lookup by a primary key.
#[async_trait::async_trait]
impl<M, S> Lookup<M, S> for &str
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    // TODO: better errors handling
    async fn lookup(&self, _parts: &mut Parts, _s: S, q: Select<M>) -> Result<Select<M>, Error> {
        let _id: Path<String> = Path::from_request_parts(_parts, &_s).await?;

        // Get the first primary key column name as a string
        let primary_key = M::PrimaryKey::iter()
            .next()
            .ok_or(Error::ImproperlyConfigured(
                "No primary key found for entity".to_string(),
            ))?;

        Ok(q.filter(primary_key.into_column().eq(_id.0)))
    }
}
