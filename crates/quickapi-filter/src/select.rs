/*
 *  The MIT License (MIT)
 *
 *  Copyright (c) 2024-2025, Peter Vrba
 *
 *  Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the "Software"), to deal
 *  in the Software without restriction, including without limitation the rights
 *  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *  copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 *  all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 *  THE SOFTWARE.
 *
 */
#![allow(unused_imports)]

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::routing::on;
use dyn_clone::DynClone;
use sea_orm::Select;
use std::marker::PhantomData;
use std::pin::Pin;

#[async_trait::async_trait]
pub trait SelectFilter<E, S, T>
where
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    /// filter_select is called to filter the select query.
    async fn filter_select(
        &self,
        parts: &mut Parts,
        state: &S,
        query: Select<E>,
    ) -> Result<Select<E>, crate::Error>;

    /// id is an optional method to return an identifier for the filter.
    /// this is useful for filter that needs to be just once in the SelectFilters.
    fn id(&self) -> Option<String> {
        None
    }
}

pub trait SelectFilterErased<E, S>: Send + Sync + DynClone
where
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn filter_select_boxed<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: Select<E>,
    ) -> Pin<Box<dyn Future<Output = Result<Select<E>, crate::Error>> + Send + 'a>>;

    /// id is an optional method to return an identifier for the filter.
    fn id(&self) -> Option<String> {
        None
    }
}

dyn_clone::clone_trait_object!(<E, S> SelectFilterErased<E, S>);

pub struct SelectModelBoxed<F, E, S, T>
where
    F: SelectFilter<E, S, T> + Send + Sync + 'static,
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    inner: F,
    _phantom: PhantomData<(E, S, T)>,
}

// Implement Clone for SelectModelBoxed
impl<F, E, S, T> Clone for SelectModelBoxed<F, E, S, T>
where
    F: SelectFilter<E, S, T> + Send + Sync + Clone + 'static,
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<F, E, S, T> SelectFilterErased<E, S> for SelectModelBoxed<F, E, S, T>
where
    F: SelectFilter<E, S, T> + Clone + Send + Sync + 'static,
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: Sync + Send + 'static,
{
    /// filter_select_boxed is called to filter the select query.
    fn filter_select_boxed<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: Select<E>,
    ) -> Pin<Box<dyn Future<Output = Result<Select<E>, crate::Error>> + Send + 'a>> {
        Box::pin(self.inner.filter_select(parts, state, query))
    }

    /// id returns the identifier of the filter if it exists.
    fn id(&self) -> Option<String> {
        self.inner.id()
    }
}

/// SelectFilters is a collection of select filters that can be applied to a select query.
pub struct SelectFilters<E, S>
where
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    pub(crate) inner: Vec<Box<dyn SelectFilterErased<E, S> + Send + Sync>>,
}

// Implement Clone for SelectFilters
impl<E, S> Clone for SelectFilters<E, S>
where
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// Clone filters by cloning each boxed filter
    fn clone(&self) -> Self {
        Self {
            inner: self
                .inner
                .iter()
                .map(|cb| dyn_clone::clone_box(&**cb))
                .collect(),
        }
    }
}

impl<E, S> SelectFilters<E, S>
where
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    // new creates a new instance of SelectFilters.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    // push adds a new filter to the SelectFilters. checks if the filter is already present.
    pub fn push<F, T>(&mut self, f: F)
    where
        F: SelectFilter<E, S, T> + Clone + Send + Sync + 'static,
        T: Sync + Send + 'static,
    {
        let boxed = Box::new(SelectModelBoxed {
            inner: f,
            _phantom: PhantomData,
        });

        // replace the filter if it already exists
        if let Some(id) = boxed.id() {
            if let Some(pos) = self
                .inner
                .iter()
                .position(|f| f.id().as_deref() == Some(&id))
            {
                self.inner[pos] = boxed;
                return; // Exit early if we replaced an existing filter
            }
        }

        self.inner.push(boxed);
    }

    // delete removes a filter from the SelectFilters by its id.
    pub fn delete<F>(&mut self, id: &str)
    where
        F: SelectFilter<E, S, ()> + Send + Sync + 'static,
    {
        self.inner.retain(|f| f.id().as_deref() != Some(id));
    }
}

impl<E, S> SelectFilterErased<E, S> for SelectFilters<E, S>
where
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn filter_select_boxed<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: Select<E>,
    ) -> Pin<Box<dyn Future<Output = Result<Select<E>, crate::Error>> + Send + 'a>> {
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
        impl<F, Fut, E, S, $($ty,)* $last> SelectFilter<E, S, ($($ty,)* $last,)> for F
        where
            E: sea_orm::EntityTrait + Send + Sync + 'static,
            S: Sync + Send + Clone + 'static,
            F: Fn(sea_orm::Select<E>, $($ty,)* $last) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = Result<sea_orm::Select<E>, crate::Error>> + Send + 'static,
            $(
                $ty: axum::extract::FromRequestParts<S> + Send + 'static,
            )*
            $last: FromRequestParts<S> + Send + 'static,
        {
            async fn filter_select(
                &self,
                parts: &mut Parts,
                state: &S,
                query: sea_orm::Select<E>,
            ) -> Result<sea_orm::Select<E>, crate::Error> {
                $(
                    let $ty = $ty::from_request_parts(parts, state).await.map_err(|_| {
                        crate::Error::NoMatch
                    })?;
                )*
                let $last = $last::from_request_parts(parts, state).await.map_err(|_| {
                    crate::Error::NoMatch
                })?;

                (self)(query, $($ty,)* $last).await
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
    pub async fn primary_key_filter<E>(
        _query: Select<E>,
        _x: axum::extract::OriginalUri,
        _y: axum::extract::OriginalUri,
    ) -> Result<Select<E>, crate::Error>
    where
        E: EntityTrait + Send + Sync + 'static,
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
