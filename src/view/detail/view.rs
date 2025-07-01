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
use crate::view::detail::DetailViewTrait;
use crate::view::handler::Handler;
use axum::Router;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::on;
use quickapi_filter::SelectFilterErased;
use quickapi_http::ModelSerializerJson;
use quickapi_http::response::{Key, Response};
use quickapi_lookup::Lookup;
use quickapi_view::{ViewTrait, as_method_filter};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_json::json;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

const DEFAULT_JSON_KEY: &str = "object";

/// DetailView is a view for displaying details of a single entity.
pub struct DetailView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    db: DatabaseConnection,
    path: String,
    method: Method,
    when: quickapi_when::WhenViews<S>,
    lookup: Arc<dyn Lookup<E, S>>,
    filters: quickapi_filter::SelectFilters<E, S>,
    ser: ModelSerializerJson<O>,
    wrap_json_key: Option<Key>,
    fallback: bool,
    _phantom: PhantomData<(E, S, O)>,
}

/// Implementing DetailView for creating a new instance.
impl<E, S, O> DetailView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub(crate) fn new(
        db: DatabaseConnection,
        path: impl AsRef<str>,
        method: Method,
        lookup: impl Lookup<E, S> + 'static,
    ) -> Self {
        Self {
            db,
            path: path.as_ref().to_string(),
            method,
            when: Default::default(),
            lookup: Arc::new(lookup),
            filters: quickapi_filter::SelectFilters::new(),
            ser: ModelSerializerJson::<O>::new(),
            wrap_json_key: Some(DEFAULT_JSON_KEY.into()),
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
        Ser: Clone + serde::Serialize + Send + Sync + 'static + From<<E as EntityTrait>::Model>,
        F: Fn(DetailView<E, S, O>) -> Result<DetailView<E, S, Ser>, Error>,
        T: Sync + Send + 'static,
    {
        let mut clone = self.clone();
        clone.when = Default::default();
        let _result = _f(clone)?;
        self.when.add_when(_when, _result);
        Ok(self)
    }

    /// with_lookup sets the lookup for the DetailView.
    pub fn with_lookup(mut self, lookup: impl Lookup<E, S> + 'static) -> Self {
        self.lookup = Arc::new(lookup);
        self
    }

    /// with_filter sets a filter for the DetailView.
    #[allow(unused_mut)]
    pub fn with_filter<F, T>(
        mut self,
        _filter: impl quickapi_filter::SelectFilter<E, S, T> + Clone + Send + Sync + 'static,
    ) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.filters.push(_filter);
        self
    }

    /// with_serializer creates a new DetailView with a specified serializer.
    pub fn with_serializer<Ser>(self) -> DetailView<E, S, Ser>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
    {
        DetailView::<E, S, Ser> {
            db: self.db,
            path: self.path,
            method: self.method,
            _phantom: PhantomData,
            when: self.when,
            lookup: self.lookup,
            filters: self.filters,
            ser: ModelSerializerJson::<Ser>::new(),
            wrap_json_key: self.wrap_json_key,
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
impl<E, S, O> DetailViewTrait<E, S> for DetailView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + From<<E as EntityTrait>::Model> + Clone + Send + Sync + 'static,
{
}

impl<E, S, O> Clone for DetailView<E, S, O>
where
    E: EntityTrait,
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
            wrap_json_key: self.wrap_json_key.clone(),
            fallback: self.fallback,
        }
    }
}

/// Implementing RouterExt for DetailView to register the router.
impl<E, S, O> quickapi_view::RouterExt<S> for DetailView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + From<<E as EntityTrait>::Model> + Clone + Send + Sync + 'static,
{
    /// register_router_with_prefix method to register the DetailView with an axum router.
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        prefix: &str,
    ) -> Result<Router<S>, quickapi_view::Error> {
        let mf = as_method_filter(&self.method)?;
        let path = format!("{}{}", prefix, self.path);

        debug!(method = self.method.to_string(), path = &path, "API detail",);

        // Register the ListView with the axum router
        Ok(router.route(&path, on(mf, Handler::new(self.clone()))))
    }
}

/// Implementing View for DetailView to render the detail view.
#[async_trait::async_trait]
impl<E, S, O> ViewTrait<S> for DetailView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + From<<E as EntityTrait>::Model> + Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        mut _parts: &mut Parts,
        _state: &S,
        _body: &bytes::Bytes,
    ) -> Result<Response, quickapi_view::Error> {
        let mut parts = _parts.clone();

        let query = self
            .filters
            .filter_select_boxed(_parts, &_state, E::find())
            .await
            .map_err(|e| quickapi_view::Error::InternalError(Box::new(e)))?;

        // prepare lookup
        let lookup = self.lookup.clone();
        let query = lookup
            .lookup(&mut parts, &_state, query.clone())
            .await
            .map_err(|e| quickapi_view::Error::InternalError(Box::new(e)))?;

        // now perform the query
        let result = query
            .clone()
            .one(&self.db)
            .await
            .map_err(|e| quickapi_view::Error::InternalError(Box::new(e)));

        let _object = result?;

        // check for result
        let Some(object) = _object else {
            return Ok(Response::new(json!({
                "error": "Not Found",
                "message": "The requested resource was not found."
            })));
        };

        let serialized = self
            .ser
            .serialize_json(object.clone())
            .map_err(|e| quickapi_view::Error::InternalError(Box::new(e)))?;

        let object = match &self.wrap_json_key {
            Some(key) => serde_json::Value::Object(serde_json::Map::from_iter(vec![(
                Into::<String>::into(key.clone()),
                serialized,
            )])),
            None => serialized,
        };

        Ok(Response::new(object))
    }

    /// get_when_views returns a vector of when views for the DetailView.
    async fn get_when_views<'a>(
        &'a self,
        _parts: &'a mut Parts,
        _state: &'a S,
    ) -> Result<Vec<&'a (dyn ViewTrait<S> + Send + Sync)>, quickapi_view::Error> {
        self.when
            .get_views(_parts, _state)
            .await
            .map_err(|e| quickapi_view::Error::InternalError(Box::new(e)))
    }

    fn has_fallback(&self) -> bool {
        self.fallback
    }
}

/// Implementing ViewWrapResultTrait for ListView to handle JSON response wrapping
impl<E, S, O> quickapi_view::ViewWrapResultTrait<S> for DetailView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + From<<E as EntityTrait>::Model> + Clone + Send + Sync + 'static,
{
    /// wrap_result_key method to set a custom key for the JSON response
    fn wrap_result_key(mut self, key: impl Into<Key>) -> Self {
        self.wrap_json_key = Some(key.into());
        self
    }

    /// no_wrap_result_key method to disable wrapping the JSON response in a key
    fn no_wrap_result_key(mut self) -> Self {
        self.wrap_json_key = None;
        self
    }

    /// get_wrap_result_key method to retrieve the key used for wrapping the JSON response
    fn get_wrap_result_key(&self) -> Option<Key> {
        self.wrap_json_key.clone()
    }
}
