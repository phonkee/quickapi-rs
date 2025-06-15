use crate::all_the_tuples;
use crate::when::When;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

/// When static condition
#[async_trait::async_trait]
impl<S> When<S, ()> for bool
where
    S: Clone + Send + Sync + 'static,
{
    async fn when(self, _parts: Parts, _state: S) -> Result<(), super::error::Error> {
        if self {
            Ok(())
        } else {
            Err(super::error::Error::NoMatch)
        }
    }
}

/// When tuple condition
#[async_trait::async_trait]
impl<S, F, Fut> When<S, ()> for F
where
    S: Clone + Send + Sync + 'static,
    F: Fn(Parts, S) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), super::error::Error>> + Send + 'static,
{
    async fn when(self, parts: Parts, state: S) -> Result<(), super::error::Error> {
        let state = state.clone();
        let mut _parts = parts.clone();

        self(parts, state.clone()).await
    }
}

/// Implementation of When trait for tuples of different types
macro_rules! impl_when_func {
    ([$($ty:ident),*], $last:ident) => {
        #[allow(non_snake_case)]
        #[async_trait::async_trait]
        impl<S, F, Fut, $($ty,)* $last> When<S, ($($ty,)* $last,)> for F
        where
            S: Clone + Send + Sync + 'static,
            $(
                $ty: FromRequestParts<S> + Send + Sync + 'static,
            )*
            $last: FromRequestParts<S> + Send + Sync + 'static,
            F: Fn(Parts, S, $($ty,)* $last) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Result<(), super::error::Error>> + Send + 'static,
        {
            async fn when(self, parts: Parts, state: S) -> Result<(), super::error::Error> {
                let state = state.clone();

                let mut _parts = parts.clone();
                $(
                    // create T1 from request parts
                    let $ty = $ty::from_request_parts(&mut _parts, &state)
                        .await
                        .map_err(|_| super::error::Error::NoMatch)?;
                )*
                let $last = $last::from_request_parts(&mut _parts, &state)
                    .await
                    .map_err(|_| super::error::Error::NoMatch)?;

                self(parts, state.clone(), $($ty,)* $last).await
            }
        }
    }
}

all_the_tuples!(impl_when_func);
