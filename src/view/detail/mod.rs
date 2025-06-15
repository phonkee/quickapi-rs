pub mod view;

pub use view::{new, new_with_method, DetailView};

#[async_trait::async_trait]
pub trait DetailViewTrait<M, S>: crate::view::ViewTrait<S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    // trait methods here
}
