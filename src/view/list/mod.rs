pub mod view;

pub use view::ListView;

pub trait ListViewTrait {
    /// list method to retrieve a list of items
    fn list<S, O>(
        &self,
        req: axum::extract::Request,
        state: S,
    ) -> Result<Vec<O>, crate::error::Error>
    where
        S: Clone + Send + Sync + 'static,
        O: serde::Serialize + Clone + Send + Sync + 'static;
}
