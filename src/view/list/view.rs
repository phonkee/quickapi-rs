#![allow(unused_mut)]

use crate::Error;
use crate::router::RouterExt;
use crate::view::filter::Filter;
use crate::view::handler::Handler;
use crate::view::when::When;
use crate::view::when::clause::Clauses;
use axum::Router;
use axum::http::Method;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, on};
use sea_orm::sea_query::ColumnSpec::Default;
use sea_orm::{Iden, Select};
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
    phantom_data: PhantomData<O>,
    // a list of filters to apply to the query
    filters: Vec<
        Arc<
            Box<
                dyn Filter<
                        S,
                        M,
                        Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send + Sync>>,
                    >,
            >,
        >,
    >,
    // when condition to apply logic
    when: Clauses<S>,
    path: String,
    method: Method,
    fallback: bool,
}

impl<M, O, S> Clone for ListView<M, O, S>
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
            when: Clauses::<S>::default(),
            phantom_data: PhantomData,
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
    pub fn new(
        path: &str,
        method: Method,
    ) -> ListView<M, <M as sea_orm::entity::EntityTrait>::Model, S> {
        ListView::<M, <M as sea_orm::entity::EntityTrait>::Model, S> {
            path: String::from(path),
            method,
            filters: Vec::new(),
            when: Clauses::<S>::default(),
            phantom_data: PhantomData,
            fallback: false,
        }
    }
}

impl<M, O, S> ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new method to create a new ListView instance
    pub fn new_with_path(_path: &str) -> Result<ListView<M, O, S>, Error> {
        Err(Error::ImproperlyConfigured("hello".to_string()))
    }
}

/// new method to create a new ListView instance
pub fn new_with_serializer<Model, Ser, State>(
    path: &str,
    method: Method,
) -> ListView<Model, Ser, State>
where
    Model: sea_orm::entity::EntityTrait,
    State: Clone + Send + Sync + 'static,
    <Model as sea_orm::entity::EntityTrait>::Model: Into<Ser>,
    Ser: serde::Serialize + Clone + Send + Sync + 'static,
{
    ListView::<Model, Ser, State> {
        path: String::from(path),
        method,
        filters: Vec::new(),
        when: Clauses::<State>::default(),
        phantom_data: PhantomData,
        fallback: false,
    }
}

/// ListView struct for handling list views of entities
impl<M, O, S> ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
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
            when: Clauses::<S>::default(),
            // when: self
            //     .when
            //     .clone()
            //     .iter()
            //     .map(|x| x.clone().with_serializer())
            //     .collect(),
            phantom_data: PhantomData,
            fallback: self.fallback,
        }
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

    /// filter method to apply a filter condition
    pub fn filter<X>(mut self, _filter: impl Filter<S, M, X>) -> Self {
        self
    }

    /// fallback method to handle fallback logic
    pub fn fallback<F>(mut self, _fallback: F) -> Self
    where
        F: FnOnce(Self) -> Result<Self, crate::error::Error>,
    {
        self.fallback = true;
        self
    }
}

impl<M, O, S> crate::view::ViewTrait<S> for ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    type Future = Pin<
        Box<
            dyn Future<Output = Result<serde_json::Value, crate::error::Error>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    // view method to handle the request
    #[allow(unused_variables)]
    fn handle_view(&self, parts: &mut Parts, _state: S) -> Self::Future {
        Box::pin(async move {
            // Here you would implement the logic to retrieve the list of items
            Ok(serde_json::json!({"message": "ListView is working!"}))
        })
    }
}

impl<M, O, S> RouterExt<S> for ListView<M, O, S>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// register_axum method to register the view with an axum router
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
