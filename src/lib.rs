use axum::response::{IntoResponse, Response};
use std::marker::PhantomData;
use std::pin::Pin;

#[derive(Clone, Default)]
pub struct ListView<M>
where
    M: sea_orm::entity::EntityTrait,
{
    _phantom: PhantomData<M>,
}

// Handler trait implementation for RequestHandler
impl<M, S> axum::handler::Handler<(), S> for ListView<M>
where
    M: sea_orm::entity::EntityTrait,
{
    // Future type for the handler
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    // Call method to handle the request
    fn call(self, _req: axum::extract::Request, _state: S) -> Self::Future {
        Box::pin(async move { (axum::http::StatusCode::OK, "hello world".to_string()).into_response() })
    }
}
