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
    async fn case_when(self, _parts: Parts, _state: S) -> Result<(), crate::Error> {
        if self {
            Ok(())
        } else {
            Err(crate::Error::NoMatchWhen)
        }
    }
}

/// When tuple condition
#[async_trait::async_trait]
impl<S, F, Fut> When<S, ()> for F
where
    S: Clone + Send + Sync + 'static,
    F: Fn(Parts, S) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), crate::Error>> + Send + 'static,
{
    async fn case_when(self, parts: Parts, state: S) -> Result<(), crate::Error> {
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
            Fut: Future<Output = Result<(), crate::Error>> + Send + 'static,
        {
            async fn case_when(self, parts: Parts, state: S) -> Result<(), crate::Error> {
                let state = state.clone();

                let mut _parts = parts.clone();
                $(
                    // create T1 from request parts
                    let $ty = $ty::from_request_parts(&mut _parts, &state)
                        .await
                        .map_err(|_| crate::Error::NoMatchWhen)?;
                )*
                let $last = $last::from_request_parts(&mut _parts, &state)
                    .await
                    .map_err(|_| crate::Error::NoMatchWhen)?;

                self(parts, state.clone(), $($ty,)* $last).await
            }
        }
    }
}

all_the_tuples!(impl_when_func);
