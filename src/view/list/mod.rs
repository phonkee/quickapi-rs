pub mod view;

use axum::http::Method;
use serde::Serialize;

pub use view::ListView;

// new ListView function that creates a new ListView instance with default serializer
pub fn new<M, S>(path: &str) -> ListView<M, S, M::Model>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as sea_orm::entity::EntityTrait>::Model: Serialize + Clone + Send + Sync + 'static,
{
    new_with_method(path, Method::GET)
}

/// new_with_method function that creates a new ListView instance with a specified HTTP method
pub fn new_with_method<M, S>(path: &str, method: Method) -> ListView<M, S, M::Model>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as sea_orm::entity::EntityTrait>::Model: Serialize + Clone + Send + Sync + 'static,
{
    ListView::<M, S, M::Model>::new(path, method)
}
