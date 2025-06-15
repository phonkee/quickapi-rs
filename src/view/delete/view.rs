#![allow(dead_code)]
use crate::view::handler::Handler;
use crate::view::{ModelViewTrait, ViewTrait};
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::request::Parts;
use axum::routing::{MethodFilter, on};
use sea_orm::Iden;
use std::marker::PhantomData;
use tracing::debug;

/// new creates a new DeleteView instance.
pub fn new<M, S>(path: impl Into<String>) -> Result<DeleteView<M, S>, Error>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    new_with_method(path, axum::http::Method::DELETE)
}

/// new_with_method creates a new DeleteView instance with a specified method.
pub fn new_with_method<M, S>(
    path: impl Into<String>,
    method: axum::http::Method,
) -> Result<DeleteView<M, S>, Error>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    Ok(DeleteView {
        path: path.into(),
        method,
        _phantom_data: PhantomData,
    })
}
/// DeleteView is a view for handling DELETE requests for a specific entity.
#[derive(Clone)]
pub struct DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    path: String,
    method: axum::http::Method,
    _phantom_data: PhantomData<(M, S)>,
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
