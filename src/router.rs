use crate::{Error, all_the_tuples};
use axum::Router;

/// RouterExt is a trait that allows for registering views with an axum router.
pub trait RouterExt<S> {
    /// register_router registers the views in the ViewSet with the given axum router.
    fn register_router(&self, router: axum::Router<S>) -> Result<axum::Router<S>, crate::Error> {
        self.register_router_with_prefix(router, "")
    }

    /// register_router_with_prefix registers the views in the ViewSet with the given axum router
    fn register_router_with_prefix(
        &self,
        router: axum::Router<S>,
        prefix: &str,
    ) -> Result<axum::Router<S>, crate::Error>;
}

macro_rules! impl_tuple {
    ([$($ty:ident),*], $last:ident) => {
        #[allow(non_snake_case)]
        impl<S, $($ty,)* $last, > RouterExt<S> for ($($ty,)* $last, )
        where
            $($ty: RouterExt<S>,)*
            $last: RouterExt<S>,
        {
            fn register_router_with_prefix(
                &self,
                router: Router<S>,
                _prefix: &str,
            ) -> Result<Router<S>, Error> {
                let ($($ty,)* $last, ) = self;
                $(
                    let router = $ty.register_router_with_prefix(router, _prefix)?;
                )*
                let router = $last.register_router_with_prefix(router, _prefix)?;
                Ok(router)
            }
        }
    };
}

all_the_tuples!(impl_tuple);
