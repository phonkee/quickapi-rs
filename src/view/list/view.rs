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
use crate::view::handler::Handler;
use axum::Router;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::on;
use quickapi_filter::{SelectFilter, SelectFilterErased};
use quickapi_http::ModelSerializerJson;
use quickapi_http::response::Response;
use quickapi_http::response::key::Key;
use quickapi_view::RouterExt;
use quickapi_view::ViewTrait;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::default::Default;
use std::marker::PhantomData;
use tracing::debug;

const DEFAULT_JSON_KEY: &str = "objects";

/// ListView is a view for displaying a list of entities.
pub struct ListView<E, S, O = <E as EntityTrait>::Model>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    db: DatabaseConnection,
    pub filters: quickapi_filter::SelectFilters<E, S>,
    when: quickapi_when::WhenViews<S>,
    path: String,
    method: Method,
    fallback: bool,
    _phantom_data: PhantomData<E>,
    ser: ModelSerializerJson<O>,
    wrap_json_key: Option<Key>,
}

/// Implementing Clone for ListView to allow cloning of the view.
/// Does not clone when conditions.
impl<E, S, O> Clone for ListView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// Custom clone implementation for ListView
    fn clone(&self) -> Self {
        ListView {
            db: self.db.clone(),
            path: self.path.clone(),
            filters: self.filters.clone(),
            when: self.when.clone(),
            _phantom_data: PhantomData,
            method: self.method.clone(),
            fallback: false,
            ser: self.ser.clone(),
            wrap_json_key: self.wrap_json_key.clone(),
        }
    }
}

/// Implementing ListView for various functionalities.
impl<E, S, O> ListView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new method to create a new ListView instance
    pub(crate) fn new(
        db: DatabaseConnection,
        path: impl Into<String>,
        method: Method,
    ) -> ListView<E, S, O> {
        ListView::<E, S, O> {
            db,
            path: path.into(),
            method,
            filters: quickapi_filter::SelectFilters::new(),
            when: quickapi_when::WhenViews::new(),
            _phantom_data: PhantomData,
            fallback: false,
            ser: ModelSerializerJson::<O>::new(),
            wrap_json_key: Some(DEFAULT_JSON_KEY.into()),
        }
    }

    /// with_fallback method to handle fallback logic
    pub fn with_fallback<F>(mut self, _fallback: bool) -> Self {
        self.fallback = true;
        self
    }

    /// when adds a condition to the DetailView.
    #[allow(unused_mut)]
    pub fn when<F, T, Ser>(
        mut self,
        _when: impl quickapi_when::When<S, T> + Clone + Send + Sync + 'static,
        _f: F,
    ) -> Result<Self, Error>
    where
        Ser: Clone + serde::Serialize + Send + Sync + 'static + From<<E as EntityTrait>::Model>,
        F: Fn(ListView<E, S, O>) -> Result<ListView<E, S, Ser>, Error>,
        T: Sync + Send + 'static,
    {
        let mut clone = self.clone();
        clone.when = Default::default();
        let mut _result = _f(clone)?;
        self.when.add_when(_when, _result);
        Ok(self)
    }

    /// with_filter method to apply a filter condition
    /// TODO: how to automatically detect T?
    pub fn with_filter<F, T>(mut self, f: F) -> Self
    where
        F: SelectFilter<E, S, T> + Clone + Send + Sync + 'static,
        T: Sync + Send + 'static,
    {
        self.filters.push(f);
        self
    }

    /// with_serializer method to set a custom serializer
    pub fn with_serializer<Ser>(self) -> ListView<E, S, Ser>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
        <E as EntityTrait>::Model: Into<Ser>,
    {
        ListView::<E, S, Ser> {
            db: self.db,
            path: self.path,
            method: self.method,
            filters: self.filters,
            when: self.when,
            _phantom_data: PhantomData,
            fallback: self.fallback,
            ser: ModelSerializerJson::<Ser>::new(),
            wrap_json_key: self.wrap_json_key,
        }
    }
}

/// Implementing RouterExt for ListView to register the router
/// This trait allows the ListView to be registered with an axum router.
impl<E, S, O> RouterExt<S> for ListView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + From<<E as sea_orm::EntityTrait>::Model> + Clone + Send + Sync + 'static,
{
    /// register_router_with_prefix method to register the ListView with an axum router
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        prefix: &str,
    ) -> Result<Router<S>, quickapi_view::Error> {
        let mf = quickapi_view::as_method_filter(&self.method)?;
        let path = format!("{}{}", prefix, self.path);

        debug!(method = self.method.to_string(), path = &path, "API list",);

        // Register the ListView with the axum router
        Ok(router.route(&path, on(mf, Handler::new(self.clone()))))
    }
}

/// Implementing ViewTrait for ListView to handle view logic
#[async_trait::async_trait]
impl<E, S, O> ViewTrait<S> for ListView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + From<<E as sea_orm::EntityTrait>::Model> + Clone + Send + Sync + 'static,
{
    /// handle_view method to process the view request
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: &S,
        _body: &bytes::Bytes,
    ) -> Result<Response, quickapi_view::Error> {
        //
        // create query first and call filters
        //
        let query = self
            .filters
            .filter_select_boxed(_parts, &_state, E::find())
            .await
            .map_err(|e| quickapi_view::Error::InternalError(Box::new(e)))?;

        // convert objects to the desired type using the serializer
        // If the serializer is not set, we use the default one.
        let objects = query
            .all(&self.db)
            .await?
            .into_iter()
            .map(|o| {
                self.ser
                    .serialize_json(o)
                    .map_err(|e| quickapi_view::Error::InternalError(Box::new(e)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // prepare array of objects
        let mut objects = serde_json::Value::Array(objects);

        // should we wrap the JSON response in a key?
        if let Some(key) = self.wrap_json_key.clone() {
            objects =
                serde_json::Value::Object(serde_json::Map::from_iter(vec![(key.into(), objects)]));
        }

        // return the response with the serialized objects
        Ok(Response::new(objects))
    }

    /// get_when_views method to retrieve views based on conditions
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

    /// has_fallback method to check if the view has a fallback (used when when conditions are not met)
    fn has_fallback(&self) -> bool {
        self.fallback
    }
}

/// Implementing ViewWrapResultTrait for ListView to handle JSON response wrapping
impl<E, S, O> quickapi_view::ViewWrapResultTrait<S> for ListView<E, S, O>
where
    E: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + From<<E as sea_orm::EntityTrait>::Model> + Clone + Send + Sync + 'static,
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
