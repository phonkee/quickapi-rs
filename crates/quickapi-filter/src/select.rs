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

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::routing::on;
use sea_orm::Select;
use std::marker::PhantomData;
use std::pin::Pin;

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait SelectFilter<M, S, T>
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
    ) -> Result<Select<M>, crate::Error>;
}

#[allow(dead_code)]
pub trait SelectFilterErased<M, S>: Send + Sync
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn filter_select_boxed<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: Select<M>,
    ) -> Pin<Box<dyn Future<Output = Result<Select<M>, crate::Error>> + Send + 'a>>;
}

pub struct SelectModelBoxed<F, M, S, T>
where
    F: SelectFilter<M, S, T> + Send + Sync + 'static,
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    inner: F,
    _phantom: PhantomData<(M, S, T)>,
}

impl<F, M, S, T> SelectFilterErased<M, S> for SelectModelBoxed<F, M, S, T>
where
    F: SelectFilter<M, S, T> + Send + Sync + 'static,
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: Sync + Send + 'static,
{
    fn filter_select_boxed<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: Select<M>,
    ) -> Pin<Box<dyn Future<Output = Result<Select<M>, crate::Error>> + Send + 'a>> {
        Box::pin(self.inner.filter_select(parts, state, query))
    }
}

/// SelectFilters is a collection of select filters that can be applied to a select query.
pub struct SelectFilters<M, S>
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    pub(crate) inner: Vec<Box<dyn SelectFilterErased<M, S> + Send + Sync>>,
}

#[allow(dead_code)]
impl<M, S> SelectFilters<M, S>
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push<F, T>(&mut self, f: F)
    where
        F: SelectFilter<M, S, T> + Send + Sync + 'static,
        T: Sync + Send + 'static,
    {
        let boxed = Box::new(SelectModelBoxed {
            inner: f,
            _phantom: PhantomData,
        });
        self.inner.push(boxed);
    }
}

impl<M, S> SelectFilterErased<M, S> for SelectFilters<M, S>
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn filter_select_boxed<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: Select<M>,
    ) -> Pin<Box<dyn Future<Output = Result<Select<M>, crate::Error>> + Send + 'a>> {
        let mut query = query.clone(); // Clone the query to avoid borrowing issues
        Box::pin(async move {
            for fut in &self.inner {
                let new_query_result = fut.filter_select_boxed(parts, state, query.clone()).await;
                match new_query_result {
                    Ok(new_query) => query = new_query,
                    Err(e) => match e {
                        crate::Error::NoMatch => {
                            // If the filter returns NoMatch, we can skip this filter
                            continue;
                        }
                        _ => {
                            // For other errors, we propagate them
                            return Err(e);
                        }
                    },
                }
            }
            Ok(query)
        })
    }
}

macro_rules! impl_select_tuples {
    ([$($ty:ident),*], $last:ident) => {
        #[async_trait::async_trait]
        #[allow(missing_docs, non_snake_case)]
        impl<F, M, S, $($ty,)* $last> SelectFilter<M, S, ($($ty,)* $last,)> for F
        where
            M: sea_orm::EntityTrait + Send + Sync + 'static,
            S: Sync + Send + Clone + 'static,
            F: Fn(sea_orm::Select<M>, $($ty,)* $last) -> Result<sea_orm::Select<M>, crate::Error> + Sync,
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
            ) -> Result<sea_orm::Select<M>, crate::Error> {
                $(
                    let $ty = $ty::from_request_parts(parts, state).await.map_err(|_| {
                        crate::Error::NoMatch
                    })?;
                )*
                let $last = $last::from_request_parts(parts, state).await.map_err(|_| {
                    crate::Error::NoMatch
                })?;

                (self)(query, $($ty,)* $last)
            }
        }
    };
}

quickapi_macro::all_the_tuples!(impl_select_tuples);

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

    // primary_key_filter filters by primary key
    pub fn primary_key_filter<M>(
        _query: Select<M>,
        _x: axum::extract::OriginalUri,
        _y: axum::extract::OriginalUri,
    ) -> Result<Select<M>, crate::Error>
    where
        M: EntityTrait + Send + Sync + 'static,
    {
        // get id query parameter
        let id = url::form_urlencoded::parse(_x.query().unwrap_or("").as_bytes())
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.parse::<i32>().unwrap_or(0))
            .unwrap();
        Ok(_query.filter(Expr::col("id").eq(id)))
    }

    #[tokio::test]
    async fn test_select_model() {
        let mut _filters = SelectFilters::<Entity, ()>::new();
        _filters.push(primary_key_filter);

        // prepare a dummy request
        let _request = axum::http::request::Request::builder()
            .method(axum::http::Method::GET)
            .uri("https://www.rust-lang.org/?id=42")
            .body(())
            .unwrap();

        // create parts from request
        let (mut parts, _body) = _request.into_parts();
        let state = ();
        let query = _filters
            .filter_select_boxed(&mut parts, &state, Entity::find())
            .await
            .unwrap();

        println!("query: {:?}", query.build(DbBackend::Postgres).to_string());
    }
}
