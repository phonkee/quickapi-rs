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
use crate::serializer::ModelSerializerJson;
use crate::view::detail::DetailViewTrait;
use crate::view::handler::Handler;
use axum::Router;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::on;
use quickapi_lookup::Lookup;
use quickapi_view::{ViewTrait, as_method_filter};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

const DEFAULT_JSON_KEY: &str = "object";

/// DetailView is a view for displaying details of a single entity.
#[allow(dead_code)]
pub struct DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    db: DatabaseConnection,
    path: String,
    method: Method,
    when: quickapi_when::WhenViews<S>,
    lookup: Arc<dyn Lookup<M, S>>,
    filters: quickapi_filter::SelectFilters<M, S>,
    ser: ModelSerializerJson<O>,
    json_key: Option<quickapi_http::response::key::Key>,
    fallback: bool,
    _phantom: PhantomData<(M, S, O)>,
}

/// Implementing DetailView for creating a new instance.
impl<M, S, O> DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub(crate) fn new(
        db: DatabaseConnection,
        path: impl AsRef<str>,
        method: Method,
        lookup: impl Lookup<M, S> + 'static,
    ) -> Self {
        Self {
            db,
            path: path.as_ref().to_string(),
            method,
            when: Default::default(),
            lookup: Arc::new(lookup),
            filters: quickapi_filter::SelectFilters::new(),
            ser: ModelSerializerJson::<O>::new(),
            json_key: Some(DEFAULT_JSON_KEY.into()),
            fallback: false,
            _phantom: PhantomData,
        }
    }

    /// when adds a condition to the DetailView.
    pub fn when<F, T, Ser>(
        mut self,
        _when: impl quickapi_when::When<S, T> + Clone + Send + Sync + 'static,
        _f: F,
    ) -> Result<Self, Error>
    where
        Ser: Clone + serde::Serialize + Send + Sync + 'static,
        F: Fn(DetailView<M, S, O>) -> Result<DetailView<M, S, Ser>, Error>,
        T: Send + Sync + 'static,
    {
        let mut clone = self.clone();
        clone.when = Default::default();
        let mut _result = _f(clone)?;
        self.when.add_when(_when, _result);
        Ok(self)
    }

    /// wrap_response_key wraps the response key for the DetailView.
    pub fn wrap_response_key(
        mut self,
        key: impl Into<Option<quickapi_http::response::key::Key>>,
    ) -> Self {
        self.json_key = key.into();
        self
    }

    /// with_lookup sets the lookup for the DetailView.
    pub fn with_lookup(mut self, lookup: impl Lookup<M, S> + 'static) -> Self {
        self.lookup = Arc::new(lookup);
        self
    }

    /// with_filter sets a filter for the DetailView.
    #[allow(unused_mut)]
    pub fn with_filter<F, T>(
        mut self,
        _filter: impl quickapi_filter::SelectFilter<M, S, T> + Clone + Send + Sync + 'static,
    ) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.filters.push(_filter);
        self
    }

    /// with_serializer creates a new DetailView with a specified serializer.
    pub fn with_serializer<Ser>(self) -> DetailView<M, S, Ser>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
    {
        DetailView::<M, S, Ser> {
            db: self.db,
            path: self.path,
            method: self.method,
            _phantom: PhantomData,
            when: self.when,
            lookup: self.lookup,
            filters: self.filters,
            ser: ModelSerializerJson::<Ser>::new(),
            json_key: self.json_key,
            fallback: self.fallback,
        }
    }

    /// with_fallback sets the fallback behavior for the DetailView.
    pub fn with_fallback(mut self, fallback: bool) -> Self {
        self.fallback = fallback;
        self
    }
}

/// Implementing DetailViewTrait for DetailView to define the detail view behavior.
impl<M, S, O> DetailViewTrait<M, S> for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
}

impl<M, S, O> Clone for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            path: self.path.clone(),
            method: self.method.clone(),
            _phantom: PhantomData,
            when: self.when.clone(),
            lookup: self.lookup.clone(),
            filters: self.filters.clone(), // TODO: Verify if this is correct
            ser: self.ser.clone(),
            json_key: self.json_key.clone(),
            fallback: self.fallback,
        }
    }
}

/// Implementing RouterExt for DetailView to register the router.
impl<M, S, O> quickapi_view::RouterExt<S> for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// register_router_with_prefix method to register the DetailView with an axum router.
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        prefix: &str,
    ) -> Result<Router<S>, quickapi_view::Error> {
        let mf = as_method_filter(&self.method)?;
        let path = format!("{}{}", prefix, self.path);

        debug!(
            method = self.method.to_string(),
            path = &path,
            "detail view",
        );

        // Register the ListView with the axum router
        Ok(router.route(&path, on(mf, Handler::new(self.clone()))))
    }
}

/// Implementing View for DetailView to render the detail view.
#[async_trait::async_trait]
impl<M, S, O> quickapi_view::ViewTrait<S> for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    fn has_fallback(&self) -> bool {
        self.fallback
    }

    async fn handle_view(
        &self,
        mut _parts: &mut Parts,
        _state: S,
        _body: bytes::Bytes,
    ) -> Result<quickapi_http::response::Response, quickapi_view::Error> {
        let lookup = self.lookup.clone();
        let _select = M::find();
        // TODO: remove unwrap() and handle errors properly
        let _select = lookup.lookup(&mut _parts, &_state, _select).await.unwrap();
        debug!("DetailView: lookup completed");
        Ok(quickapi_http::response::Response::default())
    }

    /// get_when_views returns a vector of when views for the DetailView.
    async fn get_when_views<'a>(
        &'a self,
        _parts: &'a mut Parts,
        _state: &'a S,
    ) -> Result<Vec<&'a (dyn ViewTrait<S> + Send + Sync)>, quickapi_view::Error> {
        Ok(vec![])
    }

    /// has_when_views checks if the DetailView has any when views.
    fn has_when_views(&self) -> bool {
        !self.when.is_empty()
    }
}
