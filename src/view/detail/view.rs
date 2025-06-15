use crate::serializer::ModelSerializerJson;
use crate::view::Lookup;
use crate::view::detail::DetailViewTrait;
use crate::view::handler::Handler;
use crate::view::view::ModelViewTrait;
use crate::when::{CloneWithoutWhen, WhenViews};
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::{MethodFilter, on};
use sea_orm::Iterable;
use sea_orm::{EntityTrait, Iden};
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

// new DetailView function that creates a new DetailView instance with default serializer
pub fn new<M, S>(path: &str) -> Result<DetailView<M, S, M::Model>, Error>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
{
    new_with_method(path, Method::GET)
}

/// new_with_method function that creates a new DetailView instance with a specified HTTP method
pub fn new_with_method<M, S>(
    path: &str,
    method: Method,
) -> Result<DetailView<M, S, M::Model>, Error>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
{
    // Get the first primary key column name as a string
    let primary_key = M::PrimaryKey::iter()
        .next()
        .ok_or(Error::ImproperlyConfigured(
            "No primary key found for entity".to_string(),
        ))?
        .to_string();

    Ok(DetailView::<M, S, M::Model>::new(path, method, primary_key))
}

/// DetailView is a view for displaying details of a single entity.
#[derive(Clone)]
#[allow(dead_code)]
pub struct DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    path: String,
    method: Method,
    ph: PhantomData<(M, S, O)>,
    when: WhenViews<S, Arc<dyn DetailViewTrait<M, S> + Send + Sync + 'static>>,
    lookup: Arc<dyn Lookup<M, S>>,
    filters: crate::filter::select::model::Filters,
    ser: ModelSerializerJson<O>,
}

/// Implementing CloneWithoutWhen for DetailView to clone without WhenViews.
impl<M, S, O> CloneWithoutWhen for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// clone_without_when creates a clone of the DetailView without the WhenViews.
    fn clone_without_when(&self) -> Self {
        Self {
            when: self.when.clone(),
            path: self.path.clone(),
            method: self.method.clone(),
            ph: PhantomData,
            lookup: self.lookup.clone(),
            filters: self.filters.clone(),
            ser: self.ser.clone(),
        }
    }
}

/// Implementing DetailView for creating a new instance.
impl<M, S, O> DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub fn new(path: &str, method: Method, lookup: impl Lookup<M, S> + 'static) -> Self {
        Self {
            path: path.to_owned(),
            method,
            ph: PhantomData,
            when: WhenViews::new(),
            lookup: Arc::new(lookup),
            filters: Default::default(),
            ser: ModelSerializerJson::<O>::new(),
        }
    }

    /// when adds a condition to the DetailView.
    #[allow(unused_mut)]
    pub fn when<F, T, Ser>(
        mut self,
        _when: impl crate::when::When<S, T> + Send + Sync + 'static,
        _f: F,
    ) -> Result<Self, Error>
    where
        Ser: Clone + serde::Serialize + Send + Sync + 'static,
        F: Fn(DetailView<M, S, O>) -> Result<DetailView<M, S, Ser>, Error>,
    {
        let mut _result = _f(self.clone_without_when())?;
        self.when.add_view(_when, Arc::new(_result));
        Ok(self)
    }

    /// with_lookup sets the lookup for the DetailView.
    pub fn with_lookup(mut self, lookup: impl Lookup<M, S> + 'static) -> Self {
        self.lookup = Arc::new(lookup);
        self
    }

    /// with_filter sets a filter for the DetailView.
    pub fn with_filter<F, T>(
        mut self,
        filter: impl crate::filter::select::model::Filter<M, S, T>,
    ) -> Self {
        self.filters.push(filter);
        self
    }

    /// with_serializer creates a new DetailView with a specified serializer.
    pub fn with_serializer<Ser>(&mut self) -> DetailView<M, S, Ser>
    where
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
    {
        DetailView::<M, S, Ser> {
            path: self.path.clone(),
            method: self.method.clone(),
            ph: PhantomData,
            when: self.when.clone(),
            lookup: self.lookup.clone(),
            filters: self.filters.clone(),
            ser: ModelSerializerJson::<Ser>::new(),
        }
    }
}

/// Implementing DetailViewTrait for DetailView to define the detail view behavior.
impl<M, S, O> DetailViewTrait<M, S> for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
}

/// Implementing RouterExt for DetailView to register the router.
impl<M, S, O> crate::RouterExt<S> for DetailView<M, S, O>
where
    M: EntityTrait,
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
            path = format!("{}{}", prefix, self.path),
            method = self.method.to_string(),
            "detail view",
        );

        // Register the ListView with the axum router
        Ok(router.route(
            self.path.clone().as_str(),
            on(mf, Handler::new(self.clone())),
        ))
    }
}

/// Implementing View for DetailView to render the detail view.
#[async_trait::async_trait]
impl<M, S, O> crate::view::ViewTrait<S> for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        mut _parts: &mut Parts,
        _state: S,
        _body: Body,
    ) -> Result<JsonResponse, Error> {
        let lookup = self.lookup.clone();
        let _select = M::find();
        let _select = lookup.lookup(&mut _parts, _state.clone(), _select).await?;
        debug!("DetailView: lookup completed");
        Ok(JsonResponse::default())
    }
}

/// Implementing ModelViewTrait for DetailView to define the model view behavior.
impl<M, S, O> ModelViewTrait<M, S> for DetailView<M, S, O>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
}
