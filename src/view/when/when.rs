use crate::view::when::When;
use std::pin::Pin;

impl<S> When<S> for ()
where
    S: Clone + Send + Sync + 'static,
{
    type Future = Pin<
        Box<dyn Future<Output = Result<(), crate::view::error::Error>> + Send + Sync + 'static>,
    >;
    fn when(self, _req: &mut axum::extract::Request, _state: S) -> Self::Future {
        Box::pin(async { Ok(()) })
    }
}
