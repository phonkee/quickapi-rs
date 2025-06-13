use axum::http::request::Parts;
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
        parts: Parts,
        q: Select<M>,
    ) -> impl Future<Output = Result<Select<M>, crate::error::Error>> + Send;
}

impl<M, S> Lookup<M, S> for String
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn lookup(&self, _parts: Parts, q: Select<M>) -> Result<Select<M>, crate::error::Error> {
        Ok(q)
    }
}
