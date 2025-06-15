use crate::Error;
use sea_orm::DatabaseConnection;


/// Create a new instance of QuickApi with the provided database connection.
pub fn new<S>(db: DatabaseConnection) -> QuickApi<S> {
    QuickApi::<S> {
        db,
        _marker: std::marker::PhantomData,
    }
}

/// QuickApi is the main entry point for the QuickAPI framework, providing a database connection
#[derive(Debug)]
pub struct QuickApi<S> {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) _marker: std::marker::PhantomData<S>,
}

