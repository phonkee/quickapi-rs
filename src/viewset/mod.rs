use crate::view::View;
use std::pin::Pin;

pub struct ViewSet<S> {
    path: String,
    views: Vec<Box<dyn View<S, Future = Result<serde_json::Value, crate::error::Error>>>>,
}

#[allow(unused_mut)]
impl<S> ViewSet<S> {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            views: Vec::new(),
        }
    }

    pub fn add_view<V>(mut self, _view: V) -> Self
    where
        V: View<
                S,
                Future = Pin<
                    Box<
                        dyn Future<Output = Result<serde_json::Value, crate::error::Error>>
                            + 'static,
                    >,
                >,
            >,
    {
        self
    }

    pub fn register_axum(self, router: axum::Router<()>) -> Result<axum::Router<()>, ()> {
        // Here you would register your views with the router
        Ok(router)
    }
}
