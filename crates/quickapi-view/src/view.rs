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

use dyn_clone::DynClone;

/// ViewTrait defines the behavior of a view in the application.
#[async_trait::async_trait]
pub trait ViewTrait<S>: crate::RouterExt<S> + Sync + DynClone
where
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        parts: &mut axum::http::request::Parts,
        state: S,
        body: axum::body::Body,
    ) -> Result<quickapi_http::response::Response, crate::Error>;
}

#[async_trait::async_trait]
impl<S> ViewTrait<S> for ()
where
    S: Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        _parts: &mut axum::http::request::Parts,
        _state: S,
        _body: axum::body::Body,
    ) -> Result<quickapi_http::response::Response, crate::Error> {
        Ok(quickapi_http::response::Response::default())
    }
}

/// ModelViewTrait defines the behavior of a model view in the application.
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait ModelViewTrait<M, S>: ViewTrait<S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        parts: &mut axum::http::request::Parts,
        state: S,
        body: axum::body::Body,
    ) -> Result<quickapi_http::response::Response, crate::error::Error>;
}
