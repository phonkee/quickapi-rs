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

#![allow(unused_mut)]
use crate::Error;
use crate::filter::select::ModelFilters;
use crate::router::RouterExt;
use crate::serializer::ModelSerializerJson;
use crate::view::ViewTrait;
use crate::view::handler::Handler;
use crate::view::http::as_method_filter;
use crate::view::list::ListViewTrait;
use crate::view::view::ModelViewTrait;
use crate::when::{CloneNoWhen, When, WhenViews};
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::on;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::default::Default;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

/// ListView is a view for displaying a list of entities.
pub struct ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    db: DatabaseConnection,
    filters: ModelFilters,
    // when condition to apply logic
    when: WhenViews<S>,
    path: String,
    method: Method,
    fallback: bool,
    _phantom_data: PhantomData<M>,
    ser: ModelSerializerJson<O>,
}

/// Implementing Clone for ListView to allow cloning of the view.
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
            filters: self.filters.clone(),
            when: WhenViews::new(),
            _phantom_data: PhantomData,
            method: self.method.clone(),
            fallback: false,
            ser: self.ser.clone(),
        }
    }
}

/// Implementing CloneWithoutWhen for DetailView to clone without WhenViews.
impl<M, S, O> CloneNoWhen for ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// clone_without_when creates a clone of the DetailView without the WhenViews.
    fn clone_without_when(&self) -> Self {
        Self {
            db: self.db.clone(),
            when: WhenViews::new(),
            path: self.path.clone(),
            method: self.method.clone(),
            filters: self.filters.clone(),
            _phantom_data: PhantomData,
            fallback: self.fallback,
            ser: self.ser.clone(),
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
    pub(crate) fn new(db: DatabaseConnection, path: &str, method: Method) -> ListView<M, S, O> {
        ListView::<M, S, O> {
            db,
            path: String::from(path),
            method,
            filters: Default::default(),
            when: WhenViews::new(),
            _phantom_data: PhantomData,
            fallback: false,
            ser: ModelSerializerJson::<O>::new(),
        }
    }

    /// fallback method to handle fallback logic
    pub fn fallback<F>(mut self, _fallback: F) -> Self
    where
        F: FnOnce(Self) -> Result<Self, crate::error::Error>,
    {
        self.fallback = true;
        self
    }

    /// with_filter method to apply a filter condition
    pub fn with_filter<X>(
        mut self,
        filter: impl crate::filter::select::ModelFilter<M, S, X>,
    ) -> Self {
        self.filters.push(filter);
        self
    }

    /// when adds a condition to the DetailView.
    #[allow(unused_mut)]
    pub fn when<F, T, Ser>(
        mut self,
        _when: impl When<S, T> + Send + Sync + 'static,
        _f: F,
    ) -> Result<Self, Error>
    where
        Ser: Clone + serde::Serialize + Send + Sync + 'static,
        F: Fn(ListView<M, S, O>) -> Result<ListView<M, S, Ser>, Error>,
    {
        let mut _result = _f(self.clone_without_when())?;
        self.when.add_view(_when, Arc::new(_result));
        Ok(self)
    }

    /// with_serializer method to set a custom serializer
    pub fn with_serializer<Ser>(mut self) -> ListView<M, S, Ser>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: Into<Ser>,
    {
        ListView::<M, S, Ser> {
            db: self.db.clone(),
            path: self.path,
            method: self.method.clone(),
            filters: self.filters,
            when: self.when.clone(),
            _phantom_data: PhantomData,
            fallback: self.fallback,
            ser: ModelSerializerJson::<Ser>::new(),
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
    ) -> Result<Router<S>, Error> {
        let mf = as_method_filter(&self.method)?;

        debug!(
            path = format!("{}{}", prefix, self.path),
            method = self.method.to_string(),
            "list view",
        );

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
    /// handle_view method to process the view request
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: S,
        _body: Body,
    ) -> Result<crate::response::json::Response, Error> {
        debug!("hello from ListView handle_view");

        Ok(crate::response::json::Response {
            data: serde_json::Value::Null,
            ..Default::default()
        })
    }
}

#[async_trait::async_trait]
impl<M, S, O> ModelViewTrait<M, S> for ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    // Additional methods specific to ModelViewTrait can be implemented here
}

#[async_trait::async_trait]
impl<M, S, O> ListViewTrait<M, S> for ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    // Additional methods specific to ListViewTrait can be implemented here
}
