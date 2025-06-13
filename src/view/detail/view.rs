use super::lookup::Lookup;
use crate::view::handler::Handler;
use crate::view::when::{CloneWithoutWhen, WhenViews};
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::{MethodFilter, on};
use sea_orm::EntityTrait;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

/// DetailView is a view for displaying details of a single entity.
#[derive(Clone)]
#[allow(dead_code)]
pub struct DetailView<M, S>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    path: String,
    method: Method,
    ph: PhantomData<(M, S)>,
    when: WhenViews<M, S>,
    lookup: Arc<dyn Lookup<M, S>>,
    filters: crate::filter::SelectFilters,
}

impl<M, S> CloneWithoutWhen for DetailView<M, S>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// clone_without_when creates a clone of the DetailView without the WhenViews.
    /// TODO: remove clone
    fn clone_without_when(&self) -> Self {
        Self {
            when: WhenViews::new(),
            ..self.clone()
        }
    }
}

/// Implementing DetailView for creating a new instance.
impl<M, S> DetailView<M, S>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub fn new(path: &str, method: Method, _lookup: impl Lookup<M, S> + 'static) -> Self {
        Self {
            path: path.to_owned(),
            method,
            ph: PhantomData,
            when: WhenViews::new(),
            lookup: Arc::new(_lookup),
            filters: Default::default(),
        }
    }

    /// with_lookup sets the lookup for the DetailView.
    pub fn with_lookup(mut self, lookup: impl Lookup<M, S> + 'static) -> Self {
        self.lookup = Arc::new(lookup);
        self
    }

    /// with_filter sets a filter for the DetailView.
    pub fn with_filter<F, T>(mut self, filter: impl crate::filter::SelectFilter<M, S, T>) -> Self {
        self.filters.push(Arc::new(filter));
        self
    }
}

/// Implementing View for DetailView to render the detail view.
#[async_trait::async_trait]
impl<M, S> crate::view::ViewTrait<S> for DetailView<M, S>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        mut _parts: &mut Parts,
        _state: S,
        _body: Body,
    ) -> Result<JsonResponse, crate::error::Error> {
        let lookup = self.lookup.clone();
        let _select = M::find();
        let _select = lookup.lookup(&mut _parts, _state.clone(), _select).await?;
        Ok(JsonResponse::default())
    }
}

/// Implementing RouterExt for DetailView to register the router.
impl<M, S> crate::RouterExt<S> for DetailView<M, S>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
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
