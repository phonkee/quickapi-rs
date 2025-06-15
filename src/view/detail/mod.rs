pub mod view;

pub use view::{DetailView, View};

#[async_trait::async_trait]
pub trait DetailViewTrait<M, S>: crate::view::ViewTrait<S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    // trait methods here
}
