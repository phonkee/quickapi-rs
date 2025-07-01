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

use crate::view::handler::Handler;
use axum::Router;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::on;
use quickapi_lookup::Lookup;
use quickapi_view::as_method_filter;
use quickapi_view::{Error, ViewTrait};
use quickapi_when::WhenViews;
use sea_orm::DatabaseConnection;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

/// DeleteView is a view for handling DELETE requests for a specific entity.
#[derive(Clone)]
#[allow(dead_code)]
pub struct DeleteView<E, S>
where
    E: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    db: DatabaseConnection,
    path: String,
    method: Method,
    mode: super::DeleteMode,
    when: WhenViews<S>,
    lookup: Arc<dyn Lookup<E, S>>,
    fallback: bool,
    _phantom_data: PhantomData<(E, S)>,
}

impl<E, S> DeleteView<E, S>
where
    E: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub(crate) fn new(
        db: DatabaseConnection,
        path: impl Into<String>,
        method: Method,
        lookup: impl Lookup<E, S> + 'static,
    ) -> Self {
        Self {
            db,
            path: path.into(),
            method,
            mode: Default::default(),
            when: Default::default(),
            lookup: Arc::new(lookup),
            fallback: false,
            _phantom_data: Default::default(),
        }
    }

    /// with_fallback sets the fallback for the DeleteView.
    pub fn with_fallback(mut self, fallback: bool) -> Self {
        self.fallback = fallback;
        self
    }

    /// with_lookup sets the lookup for the DeleteView.
    pub fn with_lookup(mut self, lookup: impl Lookup<E, S> + 'static) -> Self {
        self.lookup = Arc::new(lookup);
        self
    }

    /// with_mode sets the mode for the DeleteView.
    pub fn with_mode(mut self, mode: super::DeleteMode) -> Self {
        self.mode = mode;
        self
    }
}

/// Implement the ViewTrait for DeleteView
#[async_trait::async_trait]
impl<E, S> ViewTrait<S> for DeleteView<E, S>
where
    E: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: &S,
        _body: &bytes::Bytes,
    ) -> Result<quickapi_http::response::Response, quickapi_view::Error> {
        Err(quickapi_view::Error::ImproperlyConfigured(
            "nope".to_owned(),
        ))
    }
    /// get_when_views returns a list of when views for the DeleteView.
    async fn get_when_views<'a>(
        &'a self,
        _parts: &'a mut Parts,
        _state: &'a S,
    ) -> Result<Vec<&'a (dyn ViewTrait<S> + Send + Sync)>, Error> {
        self.when
            .get_views(_parts, _state)
            .await
            .map_err(|e| Error::InternalError(Box::new(e)))
    }

    fn has_fallback(&self) -> bool {
        self.fallback
    }
}

/// Implement the RouterExt trait for DeleteView
impl<E, S> quickapi_view::RouterExt<S> for DeleteView<E, S>
where
    E: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        prefix: &str,
    ) -> Result<Router<S>, quickapi_view::Error> {
        let mf = as_method_filter(&self.method)?;
        let path = format!("{}{}", prefix, self.path);

        debug!(method = self.method.to_string(), path = &path, "API delete",);

        // Register the ListView with the axum router
        Ok(router.route(&path, on(mf, Handler::new(self.clone()))))
    }
}
