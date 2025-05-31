#![allow(unused_mut)]

use crate::view::filter::Filter;
use crate::view::when::{When, WhenView};
use crate::view::{View, get};
use axum::http::Method;
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, on};
use sea_orm::Select;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

pub struct ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    phantom_data: PhantomData<O>,
    // a list of filters to apply to the query
    filters: Vec<
        Arc<
            Box<
                dyn Filter<
                        S,
                        M,
                        Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + 'static>>,
                    >,
            >,
        >,
    >,
    // when condition to apply logic
    when: Vec<WhenView<M, S, O>>,
    path: String,
    method: Method,
}

impl<M, S, O> Clone for ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        ListView {
            path: self.path.clone(),
            filters: self.filters.clone(),
            when: self.when.clone(),
            phantom_data: PhantomData,
            method: self.method.clone(),
        }
    }
}

impl<M, S> ListView<M, S, <M as sea_orm::entity::EntityTrait>::Model>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// new method to create a new ListView instance
    pub fn new(
        path: &str,
        method: Method,
    ) -> ListView<M, S, <M as sea_orm::entity::EntityTrait>::Model> {
        ListView::<M, S, <M as sea_orm::entity::EntityTrait>::Model> {
            path: String::from(path),
            method,
            filters: Vec::new(),
            when: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    /// new method to create a new ListView instance
    pub fn new_with_serializer<Model, State, Ser>(
        path: &str,
        method: Method,
    ) -> ListView<Model, State, Ser>
    where
        Model: sea_orm::entity::EntityTrait,
        State: Clone + Send + Sync + 'static,
        <Model as sea_orm::entity::EntityTrait>::Model: Into<Ser>,
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
    {
        ListView::<Model, State, Ser> {
            path: String::from(path),
            method,
            filters: Vec::new(),
            when: Vec::new(),
            phantom_data: PhantomData,
        }
    }
}

/// ListView struct for handling list views of entities
impl<M, S, O> ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
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
            when: self
                .when
                .clone()
                .iter()
                .map(|x| x.clone().with_serializer())
                .collect(),
            phantom_data: PhantomData,
        }
    }

    /// when method to conditionally apply logic
    pub fn when<F, Ser>(mut self, _when: impl When, _f: F) -> ListView<M, S, Ser>
    where
        F: FnOnce(Self) -> Result<ListView<M, S, Ser>, crate::error::Error>,
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
        <M as sea_orm::entity::EntityTrait>::Model: Into<Ser>,
    {
        // TODO: push to when vector?
        let _x = _f(self.clone());
        // self.when.push(Arc::new(Box::new(_x)));

        // Here you can implement logic to handle the `when` condition
        // For now, we just return self
        self.with_serializer()
    }

    /// filter method to apply a filter condition
    pub fn filter<X>(mut self, _filter: impl Filter<S, M, X>) -> Self {
        self
    }
}

impl<M, S, O> ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    pub fn register_axum(
        self,
        router: axum::Router<S>,
    ) -> Result<axum::Router<S>, crate::error::Error> {
        let mf: MethodFilter = self.method.clone().try_into().unwrap();
        // Register the ListView with the axum router
        Ok(router.route(self.path.clone().as_str(), on(mf, self)))
    }
}

impl<M, S, O> View<S> for ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<serde_json::Value, crate::error::Error>>>>;

    // view method to handle the request
    fn view(&self, _req: axum::extract::Request, _state: S) -> Self::Future {
        Box::pin(async move {
            // Here you would implement the logic to retrieve the list of items
            Ok(serde_json::json!({"message": "ListView is working!"}))
        })
    }
}

// Handler trait implementation for RequestHandler
impl<M, S, O> axum::handler::Handler<(), S> for ListView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Sync + Send + 'static,
{
    // Future type for the handler
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    // Call method to handle the request
    fn call(self, _req: axum::extract::Request, _state: S) -> Self::Future {
        Box::pin(
            async move { (axum::http::StatusCode::OK, "hello world".to_string()).into_response() },
        )
    }
}
