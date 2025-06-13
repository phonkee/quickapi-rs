pub mod lookup;
pub mod view;

pub use view::{DetailView, new, new_with_method};

#[async_trait::async_trait]
pub trait DetailViewTrait<M, S>: crate::view::ViewTrait<S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    // trait methods here
}
