pub mod lookup;
pub mod view;

use crate::Error;
use axum::http::Method;
use sea_orm::Iden;
use sea_orm::Iterable;
pub use view::DetailView;

// new DetailView function that creates a new DetailView instance with default serializer
pub fn new<M, S>(path: &str) -> Result<DetailView<M, S, M::Model>, Error>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
    <M as sea_orm::entity::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
{
    new_with_method(path, Method::GET)
}

/// new_with_method function that creates a new DetailView instance with a specified HTTP method
pub fn new_with_method<M, S>(
    path: &str,
    method: Method,
) -> Result<DetailView<M, S, M::Model>, Error>
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

    Ok(DetailView::<M, S, M::Model>::new(path, method, primary_key))
}
