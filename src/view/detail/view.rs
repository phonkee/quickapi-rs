use super::lookup::Lookup;
use crate::Error;
use crate::view::View;
use axum::Router;
use axum::http::Method;
use axum::http::request::Parts;
use sea_orm::EntityTrait;
use std::marker::PhantomData;
use std::pin::Pin;

/// DetailView is a view for displaying details of a single entity.
pub struct DetailView<M, S, O>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    path: String,
    method: Method,
    ph: PhantomData<(M, S, O)>,
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
        DetailView {
            path: path.to_owned(),
            method,
            ph: PhantomData,
        }
    }
}

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
    fn view(&self, _parts: &mut Parts, _state: S) -> Self::Future {
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
    fn register_router_with_prefix(
        &self,
        router: Router<S>,
        _prefix: &str,
    ) -> Result<Router<S>, Error> {
        Ok(router)
    }
}
