#![allow(unused_mut)]
use crate::filter::queryset::SelectFilters;
use crate::router::RouterExt;
use crate::serializer::ModelSerializerJson;
use crate::view::handler::Handler;
use crate::view::when::When;
use crate::view::when::clause::Clauses;
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::{MethodFilter, on};
use std::default::Default;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

pub struct ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    filters: SelectFilters,
    // when condition to apply logic
    when: Clauses<S>,
    path: String,
    method: Method,
    fallback: bool,
    phantom_data2: PhantomData<(M, O)>,
    ser: ModelSerializerJson<O>,
}

impl<M, S, O> Clone for ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// Custom clone implementation for ListView
    fn clone(&self) -> Self {
        ListView {
            path: self.path.clone(),
            filters: self.filters.clone(),
            when: Clauses::<S>::default(),
            phantom_data2: PhantomData,
            method: self.method.clone(),
            fallback: false,
            ser: self.ser.clone(),
        }
    }
}

impl<M, S, O> ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new method to create a new ListView instance
    pub fn new(path: &str, method: Method) -> ListView<M, S, O> {
        ListView::<M, S, O> {
            path: String::from(path),
            method,
            filters: Default::default(),
            when: Clauses::<S>::default(),
            phantom_data2: PhantomData,
            fallback: false,
            ser: ModelSerializerJson::<O>::new(),
        }
    }

    /// with_serializer method to set a custom serializer
    pub fn with_serializer<Ser>(mut self) -> ListView<M, S, Ser>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
        <M as sea_orm::entity::EntityTrait>::Model: Into<Ser>,
    {
        ListView::<M, S, Ser> {
            path: self.path,
            method: self.method.clone(),
            filters: self.filters,
            when: self.when.clone(),
            phantom_data2: PhantomData,
            fallback: self.fallback,
            ser: ModelSerializerJson::<Ser>::new(),
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

    /// filter method to apply a filter condition
    pub fn filter<X>(
        mut self,
        _filter: impl crate::filter::queryset::SelectFilter<M, S, X>,
    ) -> Self {
        self.filters.push(Arc::new(_filter));
        self
    }

    /// when method to conditionally apply logic
    pub fn when<'a, F, Ser, T, W>(mut self, _when: W, _f: F) -> ListView<M, S, Ser>
    where
        F: FnOnce(Self) -> Result<ListView<M, S, O>, Error>,
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
        <M as sea_orm::entity::EntityTrait>::Model: Into<Ser>,
        W: When<S, T>,
    {
        // TODO: push to when vector?
        let _x = _f(self.clone());
        // self.when.push(Arc::new(Box::new(_x)));
        // .with_serializer()

        // Here you can implement logic to handle the `when` condition
        // For now, we just return self
        self.with_serializer()
    }
}

#[async_trait::async_trait]
impl<M, S, O> crate::view::ViewTrait<S> for ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: S,
        _body: Body,
    ) -> Result<JsonResponse, Error> {
        Ok(JsonResponse {
            data: serde_json::Value::Null,
            ..Default::default()
        })
    }
}

/// Implementing RouterExt for ListView to register the router
/// This trait allows the ListView to be registered with an axum router.
impl<M, S, O> RouterExt<S> for ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// register_router_with_prefix method to register the ListView with an axum router
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        prefix: &str,
    ) -> Result<Router<S>, Error> {
        let mf: MethodFilter = self.method.clone().try_into().map_err(|e| {
            Error::InvalidMethod(format!(
                "Failed to convert method {} to MethodFilter: {}",
                self.method, e
            ))
        })?;

        debug!(
            "list view: {}{}, method: {}",
            prefix, self.path, self.method
        );

        // Register the ListView with the axum router
        Ok(router.route(
            self.path.clone().as_str(),
            on(mf, Handler::new(self.clone())),
        ))
    }
}
