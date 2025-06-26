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

use crate::view::detail::lookup::Lookup;
use crate::view::handler::Handler;
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::on;
use quickapi_view::ViewTrait;
use quickapi_view::as_method_filter;
use sea_orm::DatabaseConnection;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

/// DeleteView is a view for handling DELETE requests for a specific entity.
#[derive(Clone)]
#[allow(dead_code)]
pub struct DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    db: DatabaseConnection,
    path: String,
    method: Method,
    lookup: Arc<dyn Lookup<M, S>>,
    _phantom_data: PhantomData<(M, S)>,
}

impl<M, S> DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub fn new(
        db: DatabaseConnection,
        path: &str,
        method: Method,
        lookup: impl Lookup<M, S> + 'static,
    ) -> Self {
        Self {
            db,
            path: path.to_owned(),
            method,
            lookup: Arc::new(lookup),
            _phantom_data: Default::default(),
        }
    }

    /// with_lookup sets the lookup for the DeleteView.
    pub fn with_lookup(mut self, lookup: impl Lookup<M, S> + 'static) -> Self {
        self.lookup = Arc::new(lookup);
        self
    }
}

/// Implement the ViewTrait for DeleteView
#[async_trait::async_trait]
impl<M, S> ViewTrait<S> for DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: S,
        _body: Body,
    ) -> Result<quickapi_http::response::Response, quickapi_view::Error> {
        Err(quickapi_view::Error::ImproperlyConfigured(
            "nope".to_owned(),
        ))
    }
}

/// Implement the RouterExt trait for DeleteView
impl<M, S> quickapi_view::RouterExt<S> for DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        prefix: &str,
    ) -> Result<Router<S>, quickapi_view::Error> {
        let mf = as_method_filter(&self.method)?;

        debug!(
            path = format!("{}{}", prefix, self.path),
            method = self.method.to_string(),
            "delete view",
        );

        // Register the ListView with the axum router
        Ok(router.route(
            self.path.clone().as_str(),
            on(mf, Handler::new(self.clone())),
        ))
    }
}
