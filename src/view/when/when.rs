use crate::view::when::When;
use axum::http::request::Parts;
use sea_orm::Select;
use std::pin::Pin;

impl<S> When<S, ()> for ()
where
    S: Clone + Send + Sync + 'static,
{
    type Future =
        Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static>>;
    fn when(self, _parts: &mut Parts, _state: S) -> Self::Future {
        Box::pin(async { Ok(()) })
    }
}

pub struct WhenHeaderValue(pub String, pub String);

impl<S> When<S, axum::http::HeaderName> for WhenHeaderValue
where
    S: Clone + Send + Sync + 'static,
{
    type Future =
        Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static>>;

    fn when(self, _parts: &mut Parts, _state: S) -> Self::Future {
        Box::pin(async move { Err(super::error::Error::NoMatch) })
    }
}

// impl<S, F, R> When<S> for F
// where
//     S: Clone + Send + Sync + 'static,
//     R: Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static,
//     F: Fn(&mut Parts, S) -> R + Send + Sync + 'static,
// {
//     type Future =
//         Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static>>;
//
//     fn when(self, parts: &mut Parts, state: S) -> Self::Future {
//         let mut parts = parts.clone();
//         Box::pin(async move { self(&mut parts, state.clone()).await })
//     }
// }

impl<S, F, R> When<S, ()> for F
where
    S: Clone + Send + Sync + 'static,
    R: Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static,
    F: Fn(&mut Parts, S, ()) -> R + Send + Sync + 'static,
{
    type Future =
        Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static>>;

    fn when(self, parts: &mut Parts, state: S) -> Self::Future {
        let mut parts = parts.clone();
        Box::pin(async move { self(&mut parts, state.clone(), ()).await })
    }
}
