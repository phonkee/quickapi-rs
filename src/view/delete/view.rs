use crate::view::handler::Handler;
use crate::view::lookup::Lookup;
use crate::view::{ModelViewTrait, ViewTrait};
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::routing::{MethodFilter, on};
use sea_orm::Iden;
use sea_orm::Iterable;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;

/// new creates a new DeleteView instance.
pub fn new<M, S>(path: impl Into<String>) -> Result<DeleteView<M, S>, Error>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    new_with_method(path, Method::DELETE)
}

/// new_with_method creates a new DeleteView instance with a specified method.
pub fn new_with_method<M, S>(
    path: impl Into<String>,
    method: Method,
) -> Result<DeleteView<M, S>, Error>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    // Get the first primary key column name as a string
    let primary_key = M::PrimaryKey::iter()
        .next()
        .ok_or(Error::ImproperlyConfigured(
            "No primary key found for entity".to_string(),
        ))?
        .to_string();

    Ok(DeleteView::new(&path.into(), method, primary_key))
}

/// DeleteView is a view for handling DELETE requests for a specific entity.
#[derive(Clone)]
pub struct DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    path: String,
    method: Method,
    #[allow(dead_code)]
    lookup: Arc<dyn Lookup<M, S>>,
    _phantom_data: PhantomData<(M, S)>,
}

impl<M, S> DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new DetailView instance without serializer. It uses the model's default serializer.
    pub fn new(path: &str, method: Method, lookup: impl Lookup<M, S> + 'static) -> Self {
        Self {
            path: path.to_owned(),
            method,
            lookup: Arc::new(lookup),
            _phantom_data: Default::default(),
        }
    }
}

/// Implement the ViewTrait for DeleteView
#[async_trait::async_trait]
impl<M, S> ViewTrait<S> for DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: S,
        _body: Body,
    ) -> Result<JsonResponse, Error> {
        Err(Error::NotImplemented("nope".to_string()))
    }
}

/// Implement the ModelViewTrait for DeleteView
impl<M, S> ModelViewTrait<M, S> for DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
}

/// Implement the RouterExt trait for DeleteView
impl<M, S> crate::RouterExt<S> for DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
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
            "delete view",
        );

        // Register the ListView with the axum router
        Ok(router.route(
            self.path.clone().as_str(),
            on(mf, Handler::new(self.clone())),
        ))
    }
}
