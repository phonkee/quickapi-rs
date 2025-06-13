pub mod view;

use serde::Serialize;

pub use view::ListView;

// new ListView function that creates a new ListView instance with default serializer
pub fn new<M, S>(path: &str) -> ListView<M, <M as sea_orm::entity::EntityTrait>::Model, S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as sea_orm::entity::EntityTrait>::Model: Serialize + Clone + Send + Sync + 'static,
{
    ListView::<M, <M as sea_orm::entity::EntityTrait>::Model, S>::new(path)
}
