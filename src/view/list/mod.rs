pub mod view;

pub use view::ListView;

#[async_trait::async_trait]
pub trait ListViewTrait<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
}
