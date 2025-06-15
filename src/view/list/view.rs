#![allow(unused_mut)]
use crate::filter::model::SelectFilters;
use crate::router::RouterExt;
use crate::serializer::ModelSerializerJson;
use crate::view::ViewTrait;
use crate::view::handler::Handler;
use crate::view::list::ListViewTrait;
use crate::view::view::ModelViewTrait;
use crate::when::{CloneWithoutWhen, When, WhenViews};
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::{MethodFilter, on};
use sea_orm::EntityTrait;
use serde::Serialize;
use std::default::Default;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

// new ListView function that creates a new ListView instance with default serializer
pub fn new<M, S>(path: &str) -> ListView<M, S, M::Model>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as EntityTrait>::Model: Serialize + Clone + Send + Sync + 'static,
{
    new_with_method(path, Method::GET)
}

/// new_with_method function that creates a new ListView instance with a specified HTTP method
pub fn new_with_method<M, S>(path: &str, method: Method) -> ListView<M, S, M::Model>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as EntityTrait>::Model: Serialize + Clone + Send + Sync + 'static,
{
    ListView::<M, S, M::Model>::new(path, method)
}

/// ListView is a view for displaying a list of entities.
pub struct ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    filters: SelectFilters,
    // when condition to apply logic
    when: WhenViews<S, Arc<dyn ListViewTrait<M, S> + Send + Sync + 'static>>,
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
impl<M, S, O> CloneWithoutWhen for ListView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// clone_without_when creates a clone of the DetailView without the WhenViews.
    fn clone_without_when(&self) -> Self {
        Self {
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
    pub fn new(path: &str, method: Method) -> ListView<M, S, O> {
        ListView::<M, S, O> {
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

    /// filter method to apply a filter condition
    pub fn filter<X>(
        mut self,
        _filter: impl crate::filter::model::SelectFilter<M, S, X>,
    ) -> Self {
        self.filters.push(Arc::new(_filter));
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
        let mf: MethodFilter = self.method.clone().try_into().map_err(|e| {
            Error::InvalidMethod(format!(
                "Failed to convert method {} to MethodFilter: {}",
                self.method, e
            ))
        })?;

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
    ) -> Result<JsonResponse, Error> {
        Ok(JsonResponse {
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
