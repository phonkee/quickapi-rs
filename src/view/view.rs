use crate::RouterExt;
use crate::view::{DetailView, detail};
use axum::body::Body;
use axum::http::Method;
use axum::http::request::Parts;
use sea_orm::EntityTrait;

/// View trait for defining a view (List, Get, Delete, Update, Create)
/// S is axum state type, which can be any type that implements Send + Sync.
/// TODO: use async_trait for the future type to allow for async operations.

#[async_trait::async_trait]
pub trait ViewTrait<S>: RouterExt<S> + Sync
where
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        parts: &mut Parts,
        state: S,
        body: Body,
    ) -> Result<crate::response::JsonResponse, crate::error::Error>;
}

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait ModelViewTrait<M, S>: ViewTrait<S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
}

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
    pub fn detail<M>(&self, path: &str) -> Result<DetailView<M, S, M::Model>, crate::Error>
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
    ) -> Result<DetailView<M, S, M::Model>, crate::Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        detail::view::new_with_method(self.db.clone(), path, method)
    }
}
