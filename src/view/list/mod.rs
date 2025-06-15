pub mod view;

use crate::Error;
use axum::http::Method;
use std::marker::PhantomData;
pub use view::ListView;

#[async_trait::async_trait]
pub trait ListViewTrait<M, S>: crate::view::ViewTrait<S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
}

/// View to create detail views in the application.
pub struct View<S> {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) _marker: PhantomData<S>,
}

/// View implements methods
impl<S> View<S> {
    pub fn new<M>(&self, path: impl AsRef<str>) -> Result<ListView<M, S, M::Model>, Error>
    where
        M: sea_orm::EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as sea_orm::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        self.new_with_method(path, Method::GET)
    }

    /// new_with_method function that creates a new DetailView instance with a specified HTTP method
    pub fn new_with_method<M>(
        &self,
        path: impl AsRef<str>,
        method: Method,
    ) -> Result<ListView<M, S, M::Model>, Error>
    where
        M: sea_orm::EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as sea_orm::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        Ok(ListView::<M, S, M::Model>::new(
            self.db.clone(),
            path.as_ref(),
            method,
        ))
    }
}
