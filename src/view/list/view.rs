#![allow(unused_mut)]
use crate::filter::queryset::SelectFilters;
use crate::router::RouterExt;
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
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

pub struct ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    filters: SelectFilters,
    // when condition to apply logic
    when: Clauses<S>,
    path: String,
    method: Method,
    fallback: bool,
    phantom_data: PhantomData<O>,
    phantom_data2: PhantomData<M>,
}

impl<M, O, S> Clone for ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// Custom clone implementation for ListView
    fn clone(&self) -> Self {
        ListView {
            path: self.path.clone(),
            filters: self.filters.clone(),
            when: Clauses::<S>::default(),
            phantom_data: PhantomData,
            phantom_data2: PhantomData,
            method: self.method.clone(),
            fallback: false,
        }
    }
}

impl<M, O, S> ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new method to create a new ListView instance
    pub fn new(path: &str) -> ListView<M, O, S> {
        ListView::<M, O, S> {
            path: String::from(path),
            method: Method::GET,
            filters: Default::default(),
            when: Clauses::<S>::default(),
            phantom_data: PhantomData,
            phantom_data2: PhantomData,
            fallback: false,
        }
    }

    /// with_method method to set the HTTP method for the ListView
    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// with_serializer method to set a custom serializer
    pub fn with_serializer<Ser>(mut self) -> ListView<M, Ser, S>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
        <M as sea_orm::entity::EntityTrait>::Model: Into<Ser>,
    {
        ListView::<M, Ser, S> {
            path: self.path,
            method: self.method.clone(),
            filters: self.filters,
            when: self.when.clone(),
            phantom_data: PhantomData,
            phantom_data2: PhantomData,
            fallback: self.fallback,
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
    pub fn when<'a, F, Ser, T, W>(mut self, _when: W, _f: F) -> ListView<M, Ser, S>
    where
        F: FnOnce(Self) -> Result<ListView<M, Ser, S>, Error>,
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

/// Implementing the ViewTrait for ListView
impl<M, O, S> crate::view::ViewTrait<S> for ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    type Future =
        Pin<Box<dyn Future<Output = Result<JsonResponse, Error>> + Send + Sync + 'static>>;

    // view method to handle the request
    #[allow(unused_variables)]
    fn handle_view(&self, parts: &mut Parts, _state: S, _body: Body) -> Self::Future {
        Box::pin(async move {
            // Here you would implement the logic to retrieve the list of items
            Ok(JsonResponse {
                data: serde_json::Value::Null,
                ..Default::default()
            })
        })
    }
}

/// Implementing RouterExt for ListView to register the router
/// This trait allows the ListView to be registered with an axum router.
impl<M, O, S> RouterExt<S> for ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
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
