use super::lookup::Lookup;
use crate::view::handler::Handler;
use crate::view::when::clause::Clauses;
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, on};
use sea_orm::EntityTrait;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

/// DetailView is a view for displaying details of a single entity.
#[derive(Clone)]
pub struct DetailView<M, O, S>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    path: String,
    method: Method,
    ph: PhantomData<(M, S)>,
    ser: PhantomData<O>,
    when: Clauses<S>,
    lookup: Arc<dyn Lookup<M, S>>,
}

impl<M, O, S> DetailView<M, O, S>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub fn new(path: &str, _lookup: impl Lookup<M, S> + 'static) -> Self {
        Self {
            path: path.to_owned(),
            method: Method::GET,
            ph: PhantomData,
            ser: PhantomData,
            when: Clauses::<S>::default(),
            lookup: Arc::new(_lookup),
        }
    }

    /// with_method sets the HTTP method for the DetailView.
    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// with_lookup sets the lookup for the DetailView.
    pub fn with_lookup(mut self, lookup: impl Lookup<M, S> + 'static) -> Self {
        self.lookup = Arc::new(lookup);
        self
    }
}

/// Implementing View for DetailView to render the detail view.
impl<M, O, S> crate::view::ViewTrait<S> for DetailView<M, O, S>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    type Future = Pin<
        Box<dyn Future<Output = Result<JsonResponse, crate::error::Error>> + Send + Sync + 'static>,
    >;

    /// view method to render the detail view.
    fn handle_view(&self, _parts: &mut Parts, _state: S, _body: Body) -> Self::Future {
        Box::pin(async { Ok(JsonResponse::default()) })
    }
}

/// Implementing RouterExt for DetailView to register the router.
impl<M, O, S> crate::RouterExt<S> for DetailView<M, O, S>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// register_router_with_prefix method to register the DetailView with an axum router.
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
            "detail view: {}{}, method: {}",
            prefix, self.path, self.method
        );

        // Register the ListView with the axum router
        Ok(router.route(
            self.path.clone().as_str(),
            on(mf, Handler::new(self.clone())),
        ))
    }
}
