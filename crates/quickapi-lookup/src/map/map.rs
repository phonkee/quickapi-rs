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
use crate::{Error, Lookup, Value};
use axum::http::request::Parts;
use sea_orm::QueryFilter;
use sea_orm::prelude::Expr;
use sea_orm::{EntityTrait, Select};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;

/// PRIMARY_KEY to be used as a constant for primary key lookups.
const PRIMARY_KEY: &str = "__primary_key__";

/// LookupMap is a structure that holds a mapping of string keys to LookupValue.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Map<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    map: HashMap<String, Value>,
    _phantom_data: PhantomData<(M, S)>,
}

/// Default implementation for LookupMap.
impl<M, S> Default for Map<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            _phantom_data: PhantomData,
        }
    }
}

/// convert HashMap<String, LookupValue> to LookupMap<M, S>
impl<M, S> From<HashMap<String, Value>> for Map<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// from converts a HashMap<String, LookupValue> into a LookupMap<M, S>
    fn from(map: HashMap<String, Value>) -> Self {
        let mut result = Self::default();
        result.map = map;
        result
    }
}

/// Implementation of LookupMap.
impl<M, S> Map<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    // update updates other LookupMap with the current one.
    pub fn update(self, other: impl Into<Self>) -> Self {
        let mut result = self;
        for (key, value) in other.into().map {
            result = result.with(key, value);
        }
        // This method is a no-op in this context, as LookupMap is immutable.
        // It can be used to chain methods if needed.
        result
    }

    /// with adds a key-value pair to the LookupMap.
    pub fn with(mut self, key: impl Into<String>, value: Value) -> Self {
        self.map.insert(key.into(), value);
        self
    }
}

/// Implementation of Lookup trait for LookupMap.
#[async_trait::async_trait]
impl<M, S> Lookup<M, S> for Map<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    // lookup converts the LookupMap into a Select query based on the provided parts and state.
    async fn lookup(&self, _parts: &mut Parts, _s: &S, _q: Select<M>) -> Result<Select<M>, Error> {
        let mut _q = _q;
        for (_key, _value) in &self.map {
            // check if the key is a primary key, otherwise treat it as a regular column
            let _key = match _key.as_str() {
                PRIMARY_KEY => quickapi_model::primary_key::<M>().map_err(|_| {
                    Error::ImproperlyConfigured("Failed to get primary key for entity".to_owned())
                })?,
                _ => _key.to_owned(),
            };

            // get the column and value for the key
            let _col = M::Column::from_str(&_key).map_err(|_| {
                Error::ImproperlyConfigured("Failed to parse primary key column".to_owned())
            })?;

            // get the value from the request parts
            let _val = _value
                .get_parts_value::<M, S>(_parts, _s)
                .await
                .map_err(|e| {
                    Error::ImproperlyConfigured(format!(
                        "Failed to get value for key '{}': {}",
                        _key, e
                    ))
                })?;

            // TODO: can we do this better? more typesafe?
            // create a column expression for the entity
            let col = Expr::col(_col);

            _q = _q.filter(col.eq(_val));
        }
        Ok(_q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use sea_orm::QueryTrait;
    use sea_orm::{
        ActiveModelBehavior, DbBackend, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
        EnumIter, PrimaryKeyTrait,
    };
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

    #[tokio::test]
    #[allow(unused_mut)]
    async fn test_lookup_map() {
        let app = Router::new().route(
            "/users/{id}",
            get(async move |r: axum::extract::Request| {
                let mut r = r;
                let (mut _parts, _body) = r.into_parts();

                // prepare lookup map
                let _m: Map<Entity, ()> = Map::from(HashMap::from([(
                    "id".to_string(),
                    Value::Path("id".to_owned()),
                )]));

                let _select = _m.lookup(&mut _parts, &(), Entity::find()).await.unwrap();

                _select.build(DbBackend::Postgres).to_string()
            }),
        );

        let server = TestServer::new(app).unwrap();
        let binding = server.get("/users/123").await;
        let _response = binding.as_bytes();
        assert!(
            std::str::from_utf8(_response)
                .unwrap()
                .contains("WHERE \"id\" = '123'")
        );
    }
}
