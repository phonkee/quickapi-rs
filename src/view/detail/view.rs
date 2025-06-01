use super::lookup::Lookup;
use crate::Error;
use crate::view::View;
use crate::view::handler::Handler;
use crate::view::list::ListView;
use crate::view::when::WhenView;
use axum::Router;
use axum::http::Method;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, on};
use sea_orm::EntityTrait;
use std::marker::PhantomData;
use std::pin::Pin;
use tracing::debug;

/// DetailView is a view for displaying details of a single entity.
#[derive(Clone)]
pub struct DetailView<M, S, O>
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
    // when: Vec<WhenView<M, S, O>>,
}

impl<M, S, O> DetailView<M, S, O>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub fn new(path: &str, method: Method, _lookup: impl Lookup<M, S>) -> Self {
        Self {
            path: path.to_owned(),
            method,
            ph: PhantomData,
            ser: PhantomData,
        }
    }
}

/// Implementing View for DetailView to render the detail view.
impl<M, S, O> View<S> for DetailView<M, S, O>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
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

    /// view method to render the detail view.
    fn handle_view(&self, _parts: &mut Parts, _state: S) -> Self::Future {
        todo!()
    }
}

/// Implementing RouterExt for DetailView to register the router.
impl<M, S, O> crate::RouterExt<S> for DetailView<M, S, O>
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
