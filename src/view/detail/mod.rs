pub mod lookup;
pub mod view;

use crate::Error;
use sea_orm::Iden;
use sea_orm::Iterable;
pub use view::DetailView;

// new DetailView function that creates a new DetailView instance with default serializer
pub fn new<M, S>(path: &str) -> Result<DetailView<M, S>, Error>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as sea_orm::entity::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
{
    // Get the first primary key column name as a string
    let primary_key = M::PrimaryKey::iter()
        .next()
        .ok_or(Error::ImproperlyConfigured(
            "No primary key found for entity".to_string(),
        ))?
        .to_string();

    // TODO: check pk_name in path?
    // <M as sea_orm::entity::EntityTrait>::Model

    Ok(DetailView::<M, S>::new(path, primary_key))
}
