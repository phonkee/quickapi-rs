pub mod lookup;
pub mod view;

pub use view::DetailView;

// new DetailView function that creates a new DetailView instance with default serializer
pub fn new<M, S>(path: &str) -> DetailView<M, <M as sea_orm::entity::EntityTrait>::Model, S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as sea_orm::entity::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
{
    // get primary key column name from entity trait
    DetailView::<M, <M as sea_orm::entity::EntityTrait>::Model, S>::new(path, "id")
}
