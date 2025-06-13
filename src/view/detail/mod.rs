pub mod lookup;
pub mod view;

pub use view::DetailView;

#[async_trait::async_trait]
pub trait DetailViewTrait<M, S>: Send + Sync + 'static
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    // trait methods here
}
