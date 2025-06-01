use crate::view::when::When;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sea_orm::Select;
use std::pin::Pin;

/// When implementation for a no-op condition
impl<S> When<'_, S, ()> for ()
where
    S: Clone + Send + Sync + 'static,
{
    type Future =
        Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static>>;
    fn when(self, _parts: Parts, _state: S) -> Self::Future {
        Box::pin(async { Ok(()) })
    }
}

/// When implementation for a function that takes parts and state and returns a future
impl<'a, S, F, R> When<'a, S, f32> for F
where
    S: Clone + Send + Sync + 'static,
    R: Future<Output = Result<(), super::error::Error>> + Send + Sync + 'static,
    F: Fn(&'a Parts, S) -> R + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send + Sync>>;

    fn when(self, _parts: Parts, _state: S) -> Self::Future {
        // let _state = _state.clone();
        // #[allow(unused_mut)]
        // let mut _parts = _parts.clone();
        Box::pin(async move {
            Err(super::error::Error::NoMatch)
            // self(&mut _parts, _state).await
        })
    }
}

macro_rules! impl_when_func {
    ([$($ty:ident),*]) => {
        #[allow(non_snake_case)]
        impl<'a, S, F, R, $($ty,)*> When<'a, S, ($($ty,)*)> for F
        where
            S: Clone + Send + Sync + 'static,
            R: Future<Output = Result<(), super::error::Error>> + Send + Sync,
            F: Fn(Parts, S, $($ty,)*) -> R + Send + Sync + 'static,
            $(
                $ty: FromRequestParts<S> + Send,
            )*
        {
            type Future = Pin<Box<dyn Future<Output = Result<(), super::error::Error>> + Send>>;

            fn when(self, parts: Parts, state: S) -> Self::Future {
                let state = state.clone();

                Box::pin(async move {
                    let mut _parts = parts.clone();
                    $(
                        // create T1 from request parts
                        let $ty = $ty::from_request_parts(&mut _parts, &state)
                            .await
                            .map_err(|_| super::error::Error::NoMatch)?;
                    )*

                    self(parts, state.clone(), $($ty,)*).await
                })
            }
        }
    }
}

// implement When for functions with 1 to 8 parameters
impl_when_func!([T1]);
// impl_when_func!([T1, T2]);
// impl_when_func!([T1, T2, T3]);
// impl_when_func!([T1, T2, T3, T4]);
// impl_when_func!([T1, T2, T3, T4, T5]);
// impl_when_func!([T1, T2, T3, T4, T5, T6]);
// impl_when_func!([T1, T2, T3, T4, T5, T6, T7]);
// impl_when_func!([T1, T2, T3, T4, T5, T6, T7, T8]);
