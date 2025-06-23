/*
 * The MIT License (MIT)
 *
 * Copyright (c) 2024-2025, Peter Vrba
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

use crate::{Error};
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

quickapi_macro::all_the_tuples!(impl_tuple);
