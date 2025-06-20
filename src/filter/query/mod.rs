/*
 * The MIT License (MIT)
 *
 * Copyright (c) 2024-2025, Peter Vrba
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */
#![allow(unused_imports)]

use crate::all_the_tuples;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::routing::on;
use sea_orm::Select;
use std::marker::PhantomData;
use std::pin::Pin;

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait SelectModel<M, S, T>
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    /// filter_select is called to filter the select query.
    async fn filter_select(
        &self,
        parts: &mut Parts,
        state: &S,
        query: Select<M>,
    ) -> Result<Select<M>, crate::filter::Error>;
}

pub trait SelectModelBoxed<M, S>: Send + Sync
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn filter_select_boxed<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: Select<M>,
    ) -> Pin<Box<dyn Future<Output = Result<Select<M>, crate::filter::Error>> + Send + 'a>>;
}

pub struct SelectModelBoxedImpl<F, M, S, T>
where
    F: SelectModel<M, S, T> + Send + Sync + 'static,
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    inner: F,
    _phantom: PhantomData<(M, S, T)>,
}

impl<F, M, S, T> SelectModelBoxed<M, S> for SelectModelBoxedImpl<F, M, S, T>
where
    F: SelectModel<M, S, T> + Send + Sync + 'static,
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: Sync + Send + 'static,
{
    fn filter_select_boxed<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: Select<M>,
    ) -> Pin<Box<dyn Future<Output = Result<Select<M>, crate::filter::Error>> + Send + 'a>> {
        Box::pin(self.inner.filter_select(parts, state, query))
    }
}

pub struct SelectBoxedVecImpl<M, S>
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    pub(crate) inner: Vec<Box<dyn SelectModelBoxed<M, S> + Send + Sync>>,
}

impl<M, S> SelectBoxedVecImpl<M, S>
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push<F, T>(&mut self, f: F)
    where
        F: SelectModel<M, S, T> + Send + Sync + 'static,
        T: Sync + Send + 'static,
    {
        let boxed = Box::new(SelectModelBoxedImpl {
            inner: f,
            _phantom: PhantomData,
        });
        self.inner.push(boxed);
    }
}

macro_rules! impl_select_tuples {
    ([$($ty:ident),*], $last:ident) => {
        #[async_trait::async_trait]
        #[allow(missing_docs, non_snake_case)]
        impl<F, M, S, $($ty,)* $last> SelectModel<M, S, ($($ty,)* $last,)> for F
        where
            M: sea_orm::EntityTrait + Send + Sync + 'static,
            S: Sync + Send + Clone + 'static,
            F: Fn(sea_orm::Select<M>, $($ty,)* $last) -> Result<sea_orm::Select<M>, crate::filter::Error> + Sync,
            $(
                $ty: axum::extract::FromRequestParts<S> + Send + 'static,
            )*
            $last: FromRequestParts<S> + Send + 'static,
        {
            async fn filter_select(
                &self,
                parts: &mut Parts,
                state: &S,
                query: sea_orm::Select<M>,
            ) -> Result<sea_orm::Select<M>, crate::filter::Error> {
                $(
                    let $ty = $ty::from_request_parts(parts, state).await.map_err(|_| {
                        crate::filter::Error::NoMatch
                    })?;
                )*
                let $last = $last::from_request_parts(parts, state).await.map_err(|_| {
                    crate::filter::Error::NoMatch
                })?;

                (self)(query, $($ty,)* $last)
            }
        }
    };
}

all_the_tuples!(impl_select_tuples);

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::entity::prelude::*;
    use sea_orm::{DatabaseConnection, DbBackend, QueryTrait};
    use serde::Serialize;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    pub fn some_filter<M>(
        _query: Select<M>,
        _x: axum::extract::RawQuery,
    ) -> Result<Select<M>, crate::filter::Error>
    where
        M: sea_orm::EntityTrait + Send + Sync + 'static,
    {
        Ok(_query.filter(Expr::col("id").eq(1)))
    }

    #[tokio::test]
    async fn test_select_model() {
        let mut _filters = SelectBoxedVecImpl::<Entity, ()>::new();
        _filters.push(some_filter);
        _filters.push(some_filter);

        // prepare a dummy request
        let _request = axum::http::request::Request::builder()
            .method(axum::http::Method::GET)
            .uri("https://www.rust-lang.org/")
            .body(())
            .unwrap();

        // create parts from request
        let (mut parts, _body) = _request.into_parts();
        let state = ();
        let mut query = Entity::find();

        for filter in _filters.inner {
            query = filter
                .filter_select_boxed(&mut parts, &state, query.clone())
                .await
                .unwrap();
        }

        println!("query: {:#?}", query.build(DbBackend::Postgres).to_string());
    }
}
