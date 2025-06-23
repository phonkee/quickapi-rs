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

use crate::Error;
use crate::filter::select::ModelFilters;
use crate::serializer::ModelSerializerJson;
use crate::view::Lookup;
use crate::view::detail::DetailViewTrait;
use crate::view::handler::Handler;
use crate::view::http::as_method_filter;
use crate::view::traits::ModelViewTrait;
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::on;
use sea_orm::{DatabaseConnection, Iterable};
use sea_orm::{EntityTrait, Iden};
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

/// View to create detail views in the application.
pub struct View<S> {
    pub(crate) db: DatabaseConnection,
    pub(crate) _marker: PhantomData<S>,
}

/// View implements methods
impl<S> View<S> {
    pub fn new<M>(&self, path: impl AsRef<str>) -> Result<DetailView<M, S, M::Model>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        self.new_with_method(path, Method::GET)
    }

    /// new_with_method function that creates a new DetailView instance with a specified HTTP method
    pub fn new_with_method<M>(
        &self,
        path: impl AsRef<str>,
        method: Method,
    ) -> Result<DetailView<M, S, M::Model>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        // Get the first primary key column name as a string
        let primary_key = M::PrimaryKey::iter()
            .next()
            .ok_or(Error::ImproperlyConfigured(
                "No primary key found for entity".to_string(),
            ))?
            .to_string();

        Ok(DetailView::<M, S, M::Model>::new(
            self.db.clone(),
            path,
            method,
            primary_key,
        ))
    }
}

const DEFAULT_JSON_KEY: &str = "object";

/// DetailView is a view for displaying details of a single entity.
#[derive(Clone)]
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
    ph: PhantomData<(M, S, O)>,
    // when: WhenViews<S>,
    lookup: Arc<dyn Lookup<M, S>>,
    filters: crate::filter::select::ModelFilters<M, S>,
    ser: ModelSerializerJson<O>,
    json_key: crate::response::json::key::Key,
}

/// Implementing DetailView for creating a new instance.
impl<M, S, O> DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub fn new(
        db: DatabaseConnection,
        path: impl AsRef<str>,
        method: Method,
        lookup: impl Lookup<M, S> + 'static,
    ) -> Self {
        Self {
            db,
            path: path.as_ref().to_string(),
            method,
            ph: PhantomData,
            // when: WhenViews::new(),
            lookup: Arc::new(lookup),
            filters: ModelFilters(vec![]),
            ser: ModelSerializerJson::<O>::new(),
            json_key: DEFAULT_JSON_KEY.into(),
        }
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
        F: Fn(DetailView<M, S, O>) -> Result<DetailView<M, S, Ser>, Error>,
    {
        // let mut _result = _f(self.clone_without_when())?;
        // self.when.add_view(_when, Arc::new(_result));
        Ok(self)
    }

    /// with_json_key sets the object json key in response.
    pub fn with_json_key(mut self, key: impl Into<crate::response::json::key::Key>) -> Self {
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
        _filter: impl crate::filter::SelectModelFilter<M, S, T>,
    ) -> Self {
        // self.filters.0.push(filter);
        self
    }

    /// with_serializer creates a new DetailView with a specified serializer.
    pub fn with_serializer<Ser>(&mut self) -> DetailView<M, S, Ser>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
    {
        DetailView::<M, S, Ser> {
            db: self.db.clone(),
            path: self.path.clone(),
            method: self.method.clone(),
            ph: PhantomData,
            // when: self.when.clone(),
            lookup: self.lookup.clone(),
            filters: self.filters.clone(),
            ser: ModelSerializerJson::<Ser>::new(),
            json_key: self.json_key.clone(),
        }
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

/// Implementing RouterExt for DetailView to register the router.
impl<M, S, O> crate::RouterExt<S> for DetailView<M, S, O>
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
    ) -> Result<Router<S>, Error> {
        let mf = as_method_filter(&self.method)?;

        debug!(
            path = format!("{}{}", prefix, self.path),
            method = self.method.to_string(),
            "detail view",
        );

        // Register the ListView with the axum router
        Ok(router.route(
            self.path.clone().as_str(),
            on(mf, Handler::new(self.clone())),
        ))
    }
}

/// Implementing View for DetailView to render the detail view.
#[async_trait::async_trait]
impl<M, S, O> crate::view::ViewTrait<S> for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        mut _parts: &mut Parts,
        _state: S,
        _body: Body,
    ) -> Result<crate::response::json::Response, Error> {
        let lookup = self.lookup.clone();
        let _select = M::find();
        let _select = lookup.lookup(&mut _parts, _state.clone(), _select).await?;
        debug!("DetailView: lookup completed");
        Ok(crate::response::json::Response::default())
    }
}

/// Implementing ModelViewTrait for DetailView to define the model view behavior.
#[async_trait::async_trait]
impl<M, S, O> ModelViewTrait<M, S> for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        parts: &mut Parts,
        state: S,
        body: Body,
    ) -> Result<crate::response::json::Response, Error> {
        crate::view::ViewTrait::<S>::handle_view(self, parts, state, body).await
    }
}
