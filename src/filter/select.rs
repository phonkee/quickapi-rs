use crate::Error;
use crate::all_the_tuples;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sea_orm::Select;
use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait ModelFilter<M, S, T>: Clone + Sync + Send + 'static
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

#[async_trait::async_trait]
#[allow(missing_docs, non_snake_case)]
impl<F, M, S> ModelFilter<M, S, ()> for F
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Sync + Send + Clone + 'static,
    F: Fn(
            &mut Parts,
            S,
            Select<M>,
        ) -> Pin<Box<dyn Future<Output = Result<Select<M>, Error>> + Send>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    async fn filter_select(
        &self,
        parts: &mut Parts,
        state: S,
        query: Select<M>,
    ) -> Result<Select<M>, Error> {
        (self)(parts, state, query).await
    }
}

/// SelectFilters holds a vector of filters that can be applied to a Select query.
#[derive(Clone, Debug, Default)]
pub struct ModelFilters(pub Vec<Arc<dyn Any + Send + Sync>>);

/// Allows immutable access to the inner vector of filters.
impl Deref for ModelFilters {
    type Target = Vec<Arc<dyn Any + Send + Sync>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Allows mutable access to the inner vector of filters.
impl DerefMut for ModelFilters {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ModelFilters {
    /// push a new filter into the SelectFilters.
    pub fn push<M, S, T>(&mut self, filter: impl ModelFilter<M, S, T>)
    where
        M: sea_orm::EntityTrait + Send + Sync + 'static,
        S: Clone + Send + Sync + 'static,
    {
        self.0.push(Arc::new(filter));
    }
}

#[async_trait::async_trait]
impl<M, S, H> ModelFilter<M, S, H> for H
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    H: axum::handler::Handler<(), S>,
    S: Sync + Send + Clone + 'static,
{
    async fn filter_select(
        &self,
        _parts: &mut Parts,
        _state: S,
        query: Select<M>,
    ) -> Result<Select<M>, Error> {
        Ok(query)
    }
}

macro_rules! impl_filter_tuple {
    ([$($ty:ident),*], $last:ident) => {
        #[async_trait::async_trait]
        #[allow(missing_docs, non_snake_case)]
        impl<F, M, S, $($ty,)* $last> ModelFilter<M, S, ($($ty,)* $last,)> for F
        where
            M: sea_orm::EntityTrait + Send + Sync + 'static,
            S: Sync + Send + Clone + 'static,
            $(
                $ty: FromRequestParts<S> + Send + Sync + 'static,
            )*
            $last: FromRequestParts<S> + Send + Sync + 'static,
            F: Fn(&mut Parts, S, Select<M>, $($ty,)* $last) -> Result<Select<M>, Error> + Clone + Send + Sync + 'static,
        {
            async fn filter_select(
                &self,
                parts: &mut Parts,
                state: S,
                query: Select<M>,
            ) -> Result<Select<M>, Error> {

                $(
                let $ty = $ty::from_request_parts(parts, &state)
                    .await
                    .map_err(|_| Error::NoQueryFilterMatch)?;
                )*
                let $last = $last::from_request_parts(parts, &state)
                    .await
                    .map_err(|_| Error::NoQueryFilterMatch)?;

                (self)(parts, state, query, $($ty,)* $last,)
            }
        }
    };
}

all_the_tuples!(impl_filter_tuple);
