use axum::body::Body;
use sea_orm::Select;
use std::pin::Pin;

pub trait When: Send + Sync {
    type Future: Future<Output = Result<(), crate::view::error::Error>> + Send + Sync + 'static;

    fn when(self, req: &mut axum::http::Request<Body>) -> Self::Future;
}

impl When for () {
    type Future = Pin<
        Box<dyn Future<Output = Result<(), crate::view::error::Error>> + Send + Sync + 'static>,
    >;
    fn when(self, _req: &mut axum::http::Request<Body>) -> Self::Future {
        Box::pin(async { Err(crate::view::error::Error::NotApplied {}) })
    }
}
