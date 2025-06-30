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
use quickapi_http::Response;
use quickapi_http::serializer::ModelDeserializerJson;
use quickapi_view::{Error, ViewTrait, as_method_filter};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::marker::PhantomData;
use tracing::debug;

use quickapi_model::ModelCallbackErased;

/// CreateView is a struct that represents a view for creating new records in the database.
#[derive(Clone)]
pub struct CreateView<M, S, Ser>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    Ser: Clone
        + for<'a> serde::Deserialize<'a>
        // + Into<M::Model>
        + Sync
        + Send
        + 'static,
    <M as EntityTrait>::Model: From<Ser>,
{
    db: DatabaseConnection,
    path: String,
    method: Method,
    when: quickapi_when::WhenViews<S>,
    before_save: quickapi_model::ModelCallbacks<M, S>,
    fallback: bool,
    serializer: ModelDeserializerJson<Ser>,
    _phantom_data: PhantomData<(M, S, Ser)>,
}

/// CreateView implementation for registering the view with an axum router.
impl<M, S, Ser> quickapi_view::RouterExt<S> for CreateView<M, S, Ser>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    Ser: Clone
        + for<'a> serde::Deserialize<'a>
        // + Into<M::Model>
        + Sync
        + Send
        + 'static,
    <M as EntityTrait>::Model: From<Ser>,
{
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        prefix: &str,
    ) -> Result<Router<S>, Error> {
        let mf = as_method_filter(&self.method)?;
        let path = format!("{}{}", prefix, self.path);

        debug!(
            method = self.method.to_string(),
            path = &path,
            "create view",
        );

        // Register the ListView with the axum router
        Ok(router.route(&path, on(mf, Handler::new(self.clone()))))
    }
}

/// CreateView implementation for creating a new view for creating records in the database.
impl<M, S, Ser> CreateView<M, S, Ser>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    Ser:
        Clone + Default + serde::Serialize + for<'a> serde::Deserialize<'a> + Sync + Send + 'static,
    <M as EntityTrait>::Model: From<Ser>,
{
    // Creates a new instance of CreateView with the specified database connection, path, and method.
    pub(crate) fn new(
        db: DatabaseConnection,
        path: impl Into<String>,
        method: Method,
    ) -> Result<Self, crate::Error> {
        Ok(CreateView {
            db,
            path: path.into(),
            method,
            when: Default::default(),
            before_save: Default::default(),
            fallback: false,
            serializer: ModelDeserializerJson::<Ser>::new(),
            _phantom_data: PhantomData,
        })
    }

    /// with_serializer sets custom serializer for the CreateView.
    pub fn with_serializer<Serializer>(self) -> CreateView<M, S, Serializer>
    where
        Serializer: Clone + for<'a> serde::Deserialize<'a> + Sync + Send + 'static,
        <M as EntityTrait>::Model: From<Serializer>,
    {
        CreateView {
            db: self.db,
            path: self.path,
            method: self.method,
            when: self.when,
            before_save: self.before_save,
            serializer: ModelDeserializerJson::<Serializer>::new(),
            fallback: false,
            _phantom_data: PhantomData,
        }
    }

    /// with_before_save sets a before save handler for the CreateView.
    pub fn with_before_save<T>(
        mut self,
        before_save: impl quickapi_model::ModelCallback<M, S, T> + Clone + Send + Sync + 'static,
    ) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.before_save.push(before_save);
        self
    }

    /// remove all before save handlers from the CreateView.
    pub fn clear_before_save(mut self) -> Self {
        self.before_save.clear();
        self
    }

    /// with_fallback sets a fallback CreateView that will be used if when conditions are not met.
    pub fn with_fallback<Serializer>(mut self, fallback: bool) -> Self {
        self.fallback = fallback;
        self
    }

    // TODO: add when
}

/// Implement the ViewTrait for CreateView, which defines how the view handles requests.
#[async_trait::async_trait]
impl<M, S, Ser> ViewTrait<S> for CreateView<M, S, Ser>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    Ser: Clone + for<'a> serde::Deserialize<'a> + Sync + Send + 'static,
    <M as EntityTrait>::Model: From<Ser>,
{
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: &S,
        _body: &bytes::Bytes,
    ) -> Result<Response, Error> {
        let mut _parts = _parts.clone();

        // deserialize the body into the model (via the serializer)
        let _instance: M::Model = self
            .serializer
            .deserialize_json::<M>(_body)
            .map_err(|e| Error::InternalError(Box::new(e)))?;

        // now we need to call before_save handlers
        // TODO: add validations

        // apply before_save handlers
        let _instance = &self
            .before_save
            .call(&mut _parts, _state, _instance)
            .await
            .map_err(|e| Error::InternalError(Box::new(e)))?;

        todo!()
    }

    /// get_when_views returns a vector of views that should be executed when the CreateView is called.
    async fn get_when_views<'a>(
        &'a self,
        _parts: &'a mut Parts,
        _state: &'a S,
    ) -> Result<Vec<&'a (dyn ViewTrait<S> + Send + Sync)>, Error> {
        // Return an empty vector as CreateView does not have any when views.
        self.when
            .get_views(_parts, _state)
            .await
            .map_err(|e| Error::InternalError(Box::new(e)))
    }

    /// has_fallback returns true if the CreateView has a fallback defined (if when does not matches).
    fn has_fallback(&self) -> bool {
        self.fallback
    }
}
