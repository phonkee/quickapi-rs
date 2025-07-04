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

use axum::http::request::Parts;
use sea_orm::QueryFilter;
use sea_orm::prelude::Expr;
use sea_orm::{EntityTrait, Select};
use std::str::FromStr;

/// Lookup for primary key or other unique identifier in the database.
#[async_trait::async_trait]
pub trait Lookup<E, S>: Send + Sync
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    // lookup method filters the query based on the provided parts and state.
    async fn lookup(
        &self,
        parts: &mut Parts,
        _s: &S,
        q: Select<E>,
    ) -> Result<Select<E>, crate::Error>;
}

/// String implementation of Lookup trait. It does lookup by a primary key.
#[async_trait::async_trait]
impl<E, S> Lookup<E, S> for String
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn lookup(
        &self,
        _parts: &mut Parts,
        _s: &S,
        q: Select<E>,
    ) -> Result<Select<E>, crate::Error> {
        PrimaryKey::Path(self.clone()).lookup(_parts, _s, q).await
    }
}

/// &str implementation of Lookup trait. It does lookup by a primary key.
#[async_trait::async_trait]
impl<E, S> Lookup<E, S> for &str
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    // TODO: better errors handling
    async fn lookup(
        &self,
        _parts: &mut Parts,
        _s: &S,
        q: Select<E>,
    ) -> Result<Select<E>, crate::Error> {
        PrimaryKey::Path(ToString::to_string(&self))
            .lookup(_parts, _s, q)
            .await
    }
}

/// PrimaryKeyLookup is used to specify how to look up the primary key in the request.
pub enum PrimaryKey {
    Path(String),
    Query(String),
}

#[async_trait::async_trait]
impl<E, S> Lookup<E, S> for PrimaryKey
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn lookup(
        &self,
        _parts: &mut Parts,
        _s: &S,
        _q: Select<E>,
    ) -> Result<Select<E>, crate::Error> {
        let _pk = quickapi_model::primary_key::<E>().map_err(|err| {
            crate::Error::ImproperlyConfigured(format!(
                "Failed to get primary key for entity: {}",
                err
            ))
        })?;
        let _value = match self {
            PrimaryKey::Path(key) => super::Value::Path(key.clone())
                .get_parts_value::<E, S>(_parts, _s)
                .await?
                .clone(),
            PrimaryKey::Query(key) => super::Value::Query(key.clone())
                .get_parts_value::<E, S>(_parts, _s)
                .await?
                .clone(),
        };

        // get column from primary key string
        let col = E::Column::from_str(&_pk).map_err(|_| {
            crate::Error::ImproperlyConfigured("Failed to parse primary key column".to_owned())
        })?;

        let expr = quickapi_model::to_simple_expr(col, _value.clone()).map_err(|err| {
            crate::Error::Internal(Box::new(err))
        })?;

        Ok(_q.filter(Expr::col(col).eq(expr)))
    }
}
