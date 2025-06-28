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
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LookupMapValue {
    Path(String),
    Query(String),
}

/// Implementation of the `LookupMapValue` for extracting values from request parts.
impl LookupMapValue {
    /// from_parts extracts a value from the request parts based on the provided entity type and state.
    pub async fn get_parts_value<M, S>(
        &self,
        _parts: &mut Parts,
        _state: &S,
    ) -> Result<String, crate::Error>
    where
        M: sea_orm::EntityTrait,
        S: Clone + Send + Sync + 'static,
    {
        Ok(match self {
            LookupMapValue::Path(path) => {
                let all: Path<HashMap<String, String>> =
                    Path::from_request_parts(_parts, _state).await?;

                all.0
                    .get(path)
                    .ok_or_else(|| {
                        crate::Error::ImproperlyConfigured(format!(
                            "No value found for key '{}'",
                            &path
                        ))
                    })?
                    .clone()
            }
            LookupMapValue::Query(name) => {
                let all: axum::extract::Query<HashMap<String, String>> =
                    axum::extract::Query::from_request_parts(_parts, _state)
                        .await
                        .map_err(|_| {
                            crate::Error::ImproperlyConfigured(
                                "Failed to extract query parameters".to_owned(),
                            )
                        })?;
                // Here we would extract the primary key from the query parameters
                // Implement logic to filter the query based on the key
                all.0
                    .get(name)
                    .ok_or_else(|| {
                        crate::Error::ImproperlyConfigured(format!(
                            "No value found for key '{}'",
                            &name
                        ))
                    })?
                    .clone()
            }
        })
    }
}

#[cfg(test)]
#[allow(dead_code, unused_imports, unused_variables, unused_mut)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::extract::FromRequestParts;
    use axum::http::request::Parts;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum_test::TestServer;

    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[tokio::test]
    async fn test_path_value() {
        for i in 0..2000 {
            let app = Router::new().route(
                "/users/{id}",
                get(async move |r: axum::extract::Request| {
                    let mut r = r;
                    let (mut _parts, _body) = r.into_parts();

                    let val = LookupMapValue::Path("id".to_owned());
                    let final_val = val
                        .get_parts_value::<Entity, ()>(&mut _parts, &())
                        .await
                        .unwrap();

                    final_val
                }),
            );

            let server = TestServer::new(app).unwrap();

            let path = format!("/users/{}", i);
            let _response = server.get(&path).await;

            let result = format!("{}", i);
            assert_eq!(_response.as_bytes(), &result);
            assert_eq!(_response.status_code(), axum::http::StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_query_value() {
        for i in 0..2000 {
            let app = Router::new().route(
                "/users",
                get(async move |r: axum::extract::Request| {
                    let mut r = r;
                    let (mut _parts, _body) = r.into_parts();

                    let val = LookupMapValue::Query("id".to_owned());
                    let final_val = val
                        .get_parts_value::<Entity, ()>(&mut _parts, &())
                        .await
                        .unwrap();

                    final_val
                }),
            );

            let server = TestServer::new(app).unwrap();

            let path = format!("/users?id={}", i);
            let _response = server.get(&path).await;

            let result = format!("{}", i);
            let bytes = ToString::to_string(&result);
            assert_eq!(bytes, result);
            assert_eq!(_response.status_code(), axum::http::StatusCode::OK);
        }
    }
}
