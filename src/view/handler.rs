use crate::view::ViewTrait;
use axum::response::{IntoResponse, Response};
use std::marker::PhantomData;
use std::pin::Pin;

#[derive(Clone)]
pub(crate) struct Handler<S, V>(V, PhantomData<S>)
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
    // Only require Send, not Sync
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(self, _req: axum::extract::Request, _state: S) -> Self::Future {
        let (mut parts, body) = _req.into_parts();
        let state = _state.clone();

        Box::pin(async move {
            self.0
                .handle_view(&mut parts, state, body)
                .await
                .unwrap()
                .into_response()
        })
    }
}
