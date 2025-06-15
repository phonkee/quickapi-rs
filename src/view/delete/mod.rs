pub mod view;

use crate::Error;
use axum::http::Method;
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::{Iden, Iterable};
use std::marker::PhantomData;
pub use view::DeleteView;

/// View to create detail views in the application.
pub struct View<S> {
    pub(crate) db: DatabaseConnection,
    pub(crate) _marker: PhantomData<S>,
}

/// View implements methods
impl<S> View<S> {
    pub fn new<M>(&self, path: impl AsRef<str>) -> Result<DeleteView<M, S>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
    {
        self.new_with_method(path.as_ref(), Method::GET)
    }

    /// new_with_method function that creates a new DetailView instance with a specified HTTP method
    /// new_with_method creates a new DeleteView instance with a specified method.
    pub fn new_with_method<M>(
        &self,
        path: impl Into<String>,
        method: Method,
    ) -> Result<DeleteView<M, S>, Error>
    where
        M: sea_orm::EntityTrait,
        S: Clone + Send + Sync + 'static,
    {
        // Get the first primary key column name as a string
        let primary_key = M::PrimaryKey::iter()
            .next()
            .ok_or(Error::ImproperlyConfigured(
                "No primary key found for entity".to_string(),
            ))?
            .to_string();

        Ok(DeleteView::new(
            self.db.clone(),
            &path.into(),
            method,
            primary_key,
        ))
    }
}
