use crate::view::handler::Handler;
use crate::view::http::as_method_filter;
use crate::view::lookup::Lookup;
use crate::view::{ModelViewTrait, ViewTrait};
use crate::{Error, JsonResponse};
use axum::Router;
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::on;
use sea_orm::DatabaseConnection;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

/// DeleteView is a view for handling DELETE requests for a specific entity.
#[derive(Clone)]
pub struct DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    #[allow(dead_code)]
    db: DatabaseConnection,
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
    pub fn new(
        db: DatabaseConnection,
        path: &str,
        method: Method,
        lookup: impl Lookup<M, S> + 'static,
    ) -> Self {
        Self {
            db,
            path: path.to_owned(),
            method,
            lookup: Arc::new(lookup),
            _phantom_data: Default::default(),
        }
    }

    /// with_lookup sets the lookup for the DeleteView.
    pub fn with_lookup(mut self, lookup: impl Lookup<M, S> + 'static) -> Self {
        self.lookup = Arc::new(lookup);
        self
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
        let mf = as_method_filter(&self.method)?;

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

/// Implementing Handler for DetailView to handle requests.
impl<M, S> axum::handler::Handler<(), S> for DeleteView<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync,
{
    // Only require Send, not Sync
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(self, _req: axum::extract::Request, _state: S) -> Self::Future {
        let (mut parts, body) = _req.into_parts();
        let state = _state.clone();

        Box::pin(async move {
            self.handle_view(&mut parts, state, body)
                .await
                .unwrap()
                .into_response()
        })
    }
}
