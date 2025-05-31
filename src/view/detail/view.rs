use crate::Error;
use crate::view::View;
use axum::Router;
use axum::http::request::Parts;
use sea_orm::EntityTrait;
use std::marker::PhantomData;
use std::pin::Pin;

use super::lookup::Lookup;

/// DetailView is a view for displaying details of a single entity.
pub struct DetailView<M, S, O>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
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
    pub fn new<Model, State>(
        _lookup: impl Lookup<M, S>,
        _method: axum::http::Method,
    ) -> DetailView<Model, State, <Model as EntityTrait>::Model>
    where
        Model: EntityTrait,
        State: Clone + Send + Sync + 'static,
        <Model as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        DetailView::<Model, State, <Model as EntityTrait>::Model> { ph: PhantomData }
    }

    /// new_with_serializer creates a new DetailView with a specific serializer.
    pub fn new_with_serializer(_lookup: impl Lookup<M, S>) -> DetailView<M, S, O> {
        DetailView { ph: PhantomData }
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
