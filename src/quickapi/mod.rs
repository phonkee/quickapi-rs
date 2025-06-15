mod view;

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
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) _marker: std::marker::PhantomData<S>,
}

impl<S> QuickApi<S> {
    pub fn view(&self) -> view::View<S> {
        view::View::<S>::new(self.db.clone())
    }
}
