use axum::body::Body;

pub trait When {
    fn when(
        self,
        req: &mut axum::http::Request<Body>,
    ) -> impl Future<Output = Result<(), crate::view::error::Error>> + Send;
}

impl When for () {
    fn when(
        self,
        _req: &mut axum::http::Request<Body>,
    ) -> impl Future<Output = Result<(), crate::view::error::Error>> + Send {
        async move { Err(crate::view::error::Error::NotApplied {}) }
    }
}
