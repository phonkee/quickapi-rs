pub mod view;

pub use view::ListView;

#[async_trait::async_trait]
pub trait ListViewTrait<M, S>: crate::view::ViewTrait<S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
}
