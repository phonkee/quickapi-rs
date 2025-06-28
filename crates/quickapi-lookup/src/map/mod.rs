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
use crate::Error;
use crate::Lookup;
use axum::http::request::Parts;
use sea_orm::QueryTrait;
use sea_orm::{EntityTrait, Select};
use std::collections::HashMap;
use std::marker::PhantomData;

mod value;

pub use value::LookupMapValue;

/// LookupMap is a structure that holds a mapping of string keys to LookupValue.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct LookupMap<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    map: HashMap<String, LookupMapValue>,
    _phantom_data: PhantomData<(M, S)>,
}

/// Default implementation for LookupMap.
impl<M, S> Default for LookupMap<M, S>
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
impl<M, S> From<HashMap<String, LookupMapValue>> for LookupMap<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// from converts a HashMap<String, LookupValue> into a LookupMap<M, S>
    fn from(map: HashMap<String, LookupMapValue>) -> Self {
        let mut result = Self::new();
        result.map = map;
        result
    }
}

/// Implementation of LookupMap.
impl<M, S> LookupMap<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new empty LookupMap.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Implementation of Lookup trait for LookupMap.
#[async_trait::async_trait]
impl<M, S> Lookup<M, S> for LookupMap<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    // lookup converts the LookupMap into a Select query based on the provided parts and state.
    async fn lookup(&self, _parts: &mut Parts, _s: &S, _q: Select<M>) -> Result<Select<M>, Error> {
        for (_key, _value) in &self.map {}
        Ok(_q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use sea_orm::DbBackend;
    use sea_orm::entity::prelude::*;
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
        let mut map: LookupMap<Entity, ()> = Default::default();
        println!("{:#?}", map);

        let app = Router::new().route(
            "/users/{id}",
            get(async move |r: axum::extract::Request| {
                let mut r = r;
                let (mut _parts, _body) = r.into_parts();

                // prepare lookup map
                let _m: LookupMap<Entity, ()> = LookupMap::from(HashMap::from([(
                    "id".to_string(),
                    LookupMapValue::Path("id".to_owned()),
                )]));

                let _select = _m.lookup(&mut _parts, &(), Entity::find()).await.unwrap();

                println!(
                    "query: {:?}",
                    _select.build(DbBackend::Postgres).to_string()
                );

                "Hello, World!"
            }),
        );

        let server = TestServer::new(app).unwrap();
        let _response = server.get("/users/123").await;
    }
}
