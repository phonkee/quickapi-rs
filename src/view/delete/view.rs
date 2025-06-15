#![allow(dead_code)]
use crate::view::{ModelViewTrait, ViewTrait};
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::request::Parts;
use sea_orm::Iden;
use std::marker::PhantomData;

/// new creates a new DeleteView instance.
pub fn new<M, S>(path: impl Into<String>) -> Result<DeleteView<M, S>, crate::Error>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    Ok(DeleteView {
        path: path.into(),
        method: axum::http::Method::DELETE,
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
        _router: Router<S>,
        _prefix: &str,
    ) -> Result<Router<S>, Error> {
        Ok(_router)
    }
}
