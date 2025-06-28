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
use crate::view::handler::Handler;
use crate::view::list::ListViewTrait;
use axum::Router;
use axum::http::request::Parts;
use axum::http::{Method, StatusCode};
use axum::routing::on;
use quickapi_filter::SelectFilter;
use quickapi_http::response::key::Key;
use quickapi_view::RouterExt;
use quickapi_view::ViewTrait;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::default::Default;
use std::marker::PhantomData;
use tracing::debug;

const DEFAULT_JSON_KEY: &str = "objects";

/// ListView is a view for displaying a list of entities.
pub struct ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    db: DatabaseConnection,
    pub filters: quickapi_filter::SelectFilters<M, S>,
    // when condition to apply logic
    when: quickapi_when::WhenViews<S>,
    path: String,
    method: Method,
    fallback: bool,
    _phantom_data: PhantomData<M>,
    ser: ModelSerializerJson<O>,
    json_key: Option<Key>,
}

/// Implementing Clone for ListView to allow cloning of the view.
/// Does not clone when conditions.
impl<M, S, O> Clone for ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// Custom clone implementation for ListView
    fn clone(&self) -> Self {
        ListView {
            db: self.db.clone(),
            path: self.path.clone(),
            filters: quickapi_filter::SelectFilters::new(),
            when: quickapi_when::WhenViews::new(),
            _phantom_data: PhantomData,
            method: self.method.clone(),
            fallback: false,
            ser: self.ser.clone(),
            json_key: self.json_key.clone(),
        }
    }
}

/// Implementing ListView for various functionalities.
impl<M, S, O> ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new method to create a new ListView instance
    pub(crate) fn new(
        db: DatabaseConnection,
        path: impl Into<String>,
        method: Method,
    ) -> ListView<M, S, O> {
        ListView::<M, S, O> {
            db,
            path: path.into(),
            method,
            filters: quickapi_filter::SelectFilters::new(),
            when: quickapi_when::WhenViews::new(),
            _phantom_data: PhantomData,
            fallback: false,
            ser: ModelSerializerJson::<O>::new(),
            json_key: Some(DEFAULT_JSON_KEY.into()),
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
        _when: impl quickapi_when::When<S, T> + Send + Sync + 'static,
        _f: F,
    ) -> Result<Self, Error>
    where
        Ser: Clone + serde::Serialize + Send + Sync + 'static,
        F: Fn(ListView<M, S, O>) -> Result<ListView<M, S, Ser>, Error>,
    {
        // TODO: implement this
        // let mut _result = _f(self.clone_without_when())?;
        // self.when.add_view(_when, _result);
        Ok(self)
    }

    /// with_filter method to apply a filter condition
    /// TODO: how to automatically detect T?
    pub fn with_filter<F, T>(mut self, f: F) -> Self
    where
        F: SelectFilter<M, S, T> + Clone + Send + Sync + 'static,
        T: Sync + Send + 'static,
    {
        self.filters.push(f);
        self
    }

    /// with_json_key sets the object json key in response.
    pub fn with_json_key(mut self, key: impl Into<Option<Key>>) -> Self {
        self.json_key = key.into();
        self
    }

    /// with_serializer method to set a custom serializer
    pub fn with_serializer<Ser>(self) -> ListView<M, S, Ser>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: Into<Ser>,
    {
        ListView::<M, S, Ser> {
            db: self.db,
            path: self.path,
            method: self.method,
            filters: self.filters,
            when: self.when,
            _phantom_data: PhantomData,
            fallback: self.fallback,
            ser: ModelSerializerJson::<Ser>::new(),
            json_key: self.json_key,
        }
    }
}

/// Implementing RouterExt for ListView to register the router
/// This trait allows the ListView to be registered with an axum router.
impl<M, S, O> RouterExt<S> for ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// register_router_with_prefix method to register the ListView with an axum router
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        prefix: &str,
    ) -> Result<Router<S>, quickapi_view::Error> {
        let mf = quickapi_view::as_method_filter(&self.method)?;
        let path = format!("{}{}", prefix, self.path);

        debug!(method = self.method.to_string(), path = &path, "list view",);

        // Register the ListView with the axum router
        Ok(router.route(
            self.path.clone().as_str(),
            on(mf, Handler::new(self.clone())),
        ))
    }
}

/// Implementing ViewTrait for ListView to handle view logic
#[async_trait::async_trait]
impl<M, S, O> ViewTrait<S> for ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    fn has_fallback(&self) -> bool {
        self.fallback
    }

    /// handle_view method to process the view request
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: S,
        _body: bytes::Bytes,
    ) -> Result<quickapi_http::response::Response, quickapi_view::Error> {
        debug!("hello from ListView handle_view");

        // check if we have when conditions
        if !self.when.is_empty() {
            let mut parts_clone = _parts.clone();
            let _views = self
                .when
                .get_views(&mut parts_clone, &_state)
                .await
                .unwrap();

            // iterate over the views and handle them
            for view in _views {
                // handle the view
                let response = view
                    .handle_view(&mut _parts.clone(), _state.clone(), _body)
                    .await?;
                return Ok(response);
            }

            // we have not fallback, so we return an error
            if !self.fallback {
                return Ok(quickapi_http::response::Response {
                    data: serde_json::Value::Number(42.into()),
                    status: StatusCode::NOT_FOUND,
                    ..Default::default()
                });
            }
        }

        Ok(quickapi_http::response::Response {
            data: serde_json::Value::Number(42.into()),
            ..Default::default()
        })
    }

    /// get_when_views method to retrieve views based on conditions
    async fn get_when_views<'a>(
        &'a self,
        _parts: &'a mut Parts,
        _state: &'a S,
    ) -> Result<Vec<&'a (dyn ViewTrait<S> + Send + Sync)>, quickapi_view::Error> {
        Ok(vec![])
    }

    /// has_when_views method to check if there are any when conditions
    fn has_when_views(&self) -> bool {
        !self.when.is_empty()
    }
}
