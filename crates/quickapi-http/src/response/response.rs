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
use crate::response::partials::Partials;

#[derive(Clone, Debug)]
pub struct Response {
    data: serde_json::Value,
    status: axum::http::StatusCode,
    headers: axum::http::HeaderMap,
}

/// Default implementation for JsonResponse
impl Default for Response {
    fn default() -> Self {
        Response {
            data: serde_json::Value::Null,
            status: axum::http::StatusCode::OK,
            headers: axum::http::HeaderMap::from_iter(vec![(
                axum::http::header::CONTENT_TYPE,
                axum::http::header::HeaderValue::from_static("application/json"),
            )]),
        }
    }
}

impl Response {
    /// Creates a new JsonResponse with the given data
    pub fn new(data: serde_json::Value) -> Self {
        let mut result = Self::default();
        result.data = data;
        result
    }

    /// with_status sets the HTTP status code for the response
    pub fn with_status(mut self, status: axum::http::StatusCode) -> Self {
        self.status = status;
        self
    }

    /// with_partials adds additional partials to the response data, if not object, it will be created with given key
    pub fn with_partials<S>(mut self, key: &str, partials: &Partials<S>) -> Self {
        let mut data_map = match &self.data {
            serde_json::Value::Object(obj) => obj.clone(),
            _ => {
                let mut m = serde_json::Map::new();
                m.insert(key.into(), self.data.clone());
                m
            }
        };

        partials.update_map(&mut data_map);

        self.data = serde_json::Value::Object(data_map);

        self
    }

    /// with_header adds a header to the response
    pub fn with_header(
        mut self,
        key: impl Into<axum::http::header::HeaderName>,
        value: &str,
    ) -> Self {
        self.headers.insert(
            // TODO: remove unwrap, handle error properly
            key.into(),
            axum::http::header::HeaderValue::from_str(value).unwrap(),
        );
        self
    }
}

/// Implementing IntoResponse for JsonResponse to convert it into an axum response
impl axum::response::IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        let mut response = axum::response::Response::new(self.data.to_string().into());
        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;
        response
    }
}
