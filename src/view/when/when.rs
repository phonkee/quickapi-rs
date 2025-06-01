use crate::view::when::When;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sea_orm::Select;
use std::pin::Pin;

impl<S> When<S, ()> for ()
where
    S: Clone + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + 'static>>;
    fn when(self, _parts: &mut Parts, _state: S) -> Self::Future {
        Box::pin(async { Ok(()) })
    }
}

pub struct WhenHeaderValue(pub String, pub String);

impl<S> When<S, axum::http::HeaderName> for WhenHeaderValue
where
    S: Clone + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + 'static>>;

    fn when(self, _parts: &mut Parts, _state: S) -> Self::Future {
        Box::pin(async move { Err(super::error::Error::NoMatch) })
    }
}

impl<S, F, R> When<S, f32> for F
where
    S: Clone + Send + Sync + 'static,
    R: Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static,
    F: Fn(&mut Parts, S) -> R + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + 'static>>;

    fn when(self, _parts: &mut Parts, _state: S) -> Self::Future {
        let _state = _state.clone();
        let mut _parts = _parts.clone();
        Box::pin(async move { self(&mut _parts, _state).await })
    }
}

macro_rules! impl_when_func {
    ([$($ty:ident),*]) => {
        #[allow(non_snake_case)]
        impl<S, F, R, $($ty,)*> When<S, ($($ty,)*)> for F
        where
            S: Clone + Send + Sync + 'static,
            R: Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static,
            F: Fn(&mut Parts, S, $($ty,)*) -> R + Send + Sync + 'static,
            $(
                $ty: FromRequestParts<S> + Send + 'static,
            )*
        {
            type Future = Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + 'static>>;

            fn when(self, parts: &mut Parts, state: S) -> Self::Future {
                let mut parts = parts.clone();
                let state = state.clone();

                Box::pin(async move {
                    $(
                        // create T1 from request parts
                        let $ty = $ty::from_request_parts(&mut parts, &state)
                            .await
                            .map_err(|_| super::error::Error::NoMatch)?;
                    )*

                    self(&mut parts, state.clone(), $($ty,)*).await
                })
            }
        }
    }
}

impl_when_func!([T1]);
impl_when_func!([T1, T2]);
impl_when_func!([T1, T2, T3]);
impl_when_func!([T1, T2, T3, T4]);
impl_when_func!([T1, T2, T3, T4, T5]);
impl_when_func!([T1, T2, T3, T4, T5, T6]);
impl_when_func!([T1, T2, T3, T4, T5, T6, T7]);
impl_when_func!([T1, T2, T3, T4, T5, T6, T7, T8]);
