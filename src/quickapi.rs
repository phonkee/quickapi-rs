/// Create a new instance of QuickApi with the provided database connection.
pub fn new<S>(db: sea_orm::DatabaseConnection) -> QuickApi<S> {
    QuickApi::<S> {
        db,
        _marker: std::marker::PhantomData,
    }
}

/// QuickApi is the main entry point for the QuickAPI framework, providing a database connection
#[derive(Debug)]
#[allow(dead_code)]
pub struct QuickApi<S> {
    /// db is the database connection used by the QuickAPI framework.
    pub(crate) db: sea_orm::DatabaseConnection,
    /// _marker is a marker type to ensure that QuickApi can be used with different state types.
    pub(crate) _marker: std::marker::PhantomData<S>,
}

/// QuickApi implements methods to create views in the application.
impl<S> QuickApi<S> {
    /// view returns object to create views in the application.
    /// view is single endpoint in api.
    pub fn view(&self) -> crate::view::View<S> {
        crate::view::View::<S>::new(self.db.clone())
    }
}
