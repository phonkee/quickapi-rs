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
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::http::request::Parts;
use quickapi_view::Error;
use sea_orm::QueryFilter;
use sea_orm::prelude::Expr;
use sea_orm::{EntityTrait, Select};
use std::collections::HashMap;
use std::str::FromStr;

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
    ) -> Result<Select<M>, quickapi_view::Error>;
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
    ) -> Result<Select<M>, quickapi_view::Error> {
        PrimaryKeyLookup::Path(self.clone())
            .lookup(_parts, _s, q)
            .await
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
    async fn lookup(
        &self,
        _parts: &mut Parts,
        _s: S,
        q: Select<M>,
    ) -> Result<Select<M>, quickapi_view::Error> {
        PrimaryKeyLookup::Path(ToString::to_string(&self))
            .lookup(_parts, _s, q)
            .await
    }
}

/// PrimaryKeyLookup is used to specify how to look up the primary key in the request.
pub enum PrimaryKeyLookup {
    Path(String),
    Query(String),
}

#[async_trait::async_trait]
impl<M, S> Lookup<M, S> for PrimaryKeyLookup
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    async fn lookup(&self, _parts: &mut Parts, _s: S, _q: Select<M>) -> Result<Select<M>, Error> {
        let _pk = quickapi_model::primary_key::<M>().map_err(|err| {
            Error::ImproperlyConfigured(format!("Failed to get primary key for entity: {}", err))
        })?;
        let _value = match self {
            PrimaryKeyLookup::Path(key) => {
                let all: Path<HashMap<String, String>> =
                    Path::from_request_parts(_parts, &_s).await?;

                all.0
                    .get(key)
                    .ok_or_else(|| {
                        Error::ImproperlyConfigured(format!("No value found for key '{}'", &key))
                    })?
                    .clone()
            }
            PrimaryKeyLookup::Query(key) => {
                let all: axum::extract::Query<HashMap<String, String>> =
                    axum::extract::Query::from_request_parts(_parts, &_s)
                        .await
                        .map_err(|_| {
                            Error::ImproperlyConfigured(
                                "Failed to extract query parameters".to_owned(),
                            )
                        })?;
                // Here we would extract the primary key from the query parameters
                // Implement logic to filter the query based on the key
                all.0
                    .get(key)
                    .ok_or_else(|| {
                        Error::ImproperlyConfigured(format!("No value found for key '{}'", &key))
                    })?
                    .clone()
            }
        };

        let col = M::Column::from_str(&_pk).map_err(|_| {
            Error::ImproperlyConfigured("Failed to parse primary key column".to_owned())
        })?;

        Ok(_q.filter(Expr::col(col).eq(_value)))
    }
}
