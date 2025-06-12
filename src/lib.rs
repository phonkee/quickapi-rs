#![allow(dead_code, unused_imports)]
mod error;
pub mod router;
pub mod view;
pub mod viewset;

pub use error::Error;
pub use router::RouterExt;
use std::marker::PhantomData;
use std::pin::Pin;

pub use viewset::ViewSet;

pub struct ViewFuture<S> {
    pub(crate) inner: Pin<Box<dyn Future<Output = Result<serde_json::Value, Error>> + Send + Sync>>,
    pub(crate) _phantom: PhantomData<S>,
}

impl<S> From<Pin<Box<dyn Future<Output = Result<serde_json::Value, Error>> + Send + Sync>>>
    for ViewFuture<S>
{
    fn from(
        future: Pin<Box<dyn Future<Output = Result<serde_json::Value, Error>> + Send + Sync>>,
    ) -> Self {
        ViewFuture {
            inner: future,
            _phantom: PhantomData,
        }
    }
}
