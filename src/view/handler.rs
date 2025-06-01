use crate::view::ViewTrait;
use axum::response::{IntoResponse, Response};
use sea_orm::Iden;
use std::marker::PhantomData;
use std::pin::Pin;

#[derive(Clone)]
pub struct Handler<S, V>(V, PhantomData<S>)
where
    V: ViewTrait<S> + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static;

impl<S, V> Handler<S, V>
where
    V: ViewTrait<S> + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new Handler instance with the given view and state.
    pub fn new(view: V) -> Self {
        Self(view, PhantomData)
    }
}

/// Implementing Handler for DetailView to handle requests.
impl<S, V> axum::handler::Handler<(), S> for Handler<S, V>
where
    V: ViewTrait<S> + Clone + Send + Sync,
    S: Clone + Send + Sync,
{
    // Future type for the handler
    type Future = Pin<Box<dyn Future<Output = Response> + Send + Sync + 'static>>;

    #[allow(unused_variables, unused_mut)]
    fn call(self, _req: axum::extract::Request, _state: S) -> Self::Future {
        let (mut parts, body) = _req.into_parts();

        let state = _state.clone();

        Box::pin(async move {
            (axum::http::status::StatusCode::OK, "hello".to_string()).into_response()
        })
    }
}
