use crate::Error;
use crate::all_the_tuples;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sea_orm::Select;

#[async_trait::async_trait]
pub trait SelectFilter<M, S, T>: Sync
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn filter_queryset(
        &self,
        parts: &mut Parts,
        state: S,
        query: Select<M>,
    ) -> Result<Select<M>, Error>;
}

#[async_trait::async_trait]
impl<M, S, H> SelectFilter<M, S, H> for H
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    H: axum::handler::Handler<(), S>,
    S: Sync + Send + Clone + 'static,
{
    async fn filter_queryset(
        &self,
        _parts: &mut Parts,
        _state: S,
        query: Select<M>,
    ) -> Result<Select<M>, Error> {
        Ok(query)
    }
}

#[async_trait::async_trait]
impl<F, M, S, T> SelectFilter<M, S, (T,)> for F
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Sync + Send + Clone + 'static,
    T: FromRequestParts<S> + Send + Sync + 'static,
    F: Fn(&mut Parts, S, Select<M>, T) -> Result<Select<M>, Error> + Send + Sync + 'static,
{
    async fn filter_queryset(
        &self,
        parts: &mut Parts,
        state: S,
        query: Select<M>,
    ) -> Result<Select<M>, Error> {
        let t = T::from_request_parts(parts, &state)
            .await
            .map_err(|_| Error::NoQueryFilterMatch)?;
        (self)(parts, state, query, t)
    }
}

macro_rules! impl_filter_tuple {
    ([$($ty:ident),*], $last:ident) => {
        // #[async_trait::async_trait]
        // #[allow(non_snake_case)]
        // #[allow(missing_docs)]
        // impl<F, M, S, $($ty,)*> SelectFilter<M, S, ($($ty,)*)> for F
        // where
        //     M: sea_orm::EntityTrait + Send + Sync + 'static,
        //     S: Sync + Send + Clone + 'static,
        //     $(
        //         $ty: FromRequestParts<S> + Send + Sync + 'static,
        //     )*
        //     F: Fn(&mut Parts, S, Select<M>, $($ty,)*) -> Result<Select<M>, Error> + Send + Sync + 'static,
        // {
        //     async fn filter_queryset(
        //         &self,
        //         parts: &mut Parts,
        //         state: S,
        //         query: Select<M>,
        //     ) -> Result<Select<M>, Error> {
        //
        //         $(
        //         let $ty = $ty::from_request_parts(parts, &state)
        //             .await
        //             .map_err(|_| Error::NoQueryFilterMatch)?;
        //         )*
        //
        //         (self)(parts, state, query, $($ty,)*)
        //     }
        // }
    };
}

all_the_tuples!(impl_filter_tuple);
