use crate::Error;
use crate::view::DetailView;
use axum::http::Method;
use sea_orm::EntityTrait;

/// View provides methods to create and manage views in the application.
pub struct View<S> {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) _marker: std::marker::PhantomData<S>,
}

/// View implements methods
impl<S> View<S> {
    /// Create a new View instance with the provided database connection.
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self {
            db,
            _marker: std::marker::PhantomData,
        }
    }

    /// detail creates a new DetailView for the specified path using the GET method.
    pub fn detail<M>(&self, path: &str) -> Result<DetailView<M, S, M::Model>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        self.detail_with_method(path, Method::GET)
    }

    /// detail_with_method creates a new DetailView for the specified path using the provided HTTP method.
    pub fn detail_with_method<M>(
        &self,
        path: impl AsRef<str>,
        method: Method,
    ) -> Result<DetailView<M, S, M::Model>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        crate::view::detail::view::new_with_method(self.db.clone(), path, method)
    }
}
