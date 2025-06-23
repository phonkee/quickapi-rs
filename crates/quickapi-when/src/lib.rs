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
#![allow(dead_code, unused_imports, unused_mut)]
use axum::http::request::Parts;
use std::marker::PhantomData;
use std::pin::Pin;

/// When trait for defining conditions that must be met before executing a view
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait When<S, T>: Send
where
    S: Clone + Send,
{
    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    async fn when(&self, _parts: &mut Parts, _state: &S) -> Result<(), quickapi_view::Error>;
}

#[allow(dead_code)]
pub(crate) trait WhenErased<S>: Send
where
    S: Clone + Send,
{
    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    fn when<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
    ) -> Pin<Box<dyn Future<Output = Result<(), quickapi_view::Error>> + Send + 'a>>;
}

pub struct WhenBoxed<F, S, T>
where
    F: When<S, T> + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    inner: F,
    _phantom: PhantomData<(S, T)>,
}

impl<F, S, T> WhenErased<S> for WhenBoxed<F, S, T>
where
    F: When<S, T> + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: Send + 'static,
{
    fn when<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
    ) -> Pin<Box<dyn Future<Output = Result<(), quickapi_view::Error>> + Send + 'a>> {
        Box::pin(self.inner.when(parts, state))
    }
}

pub(crate) struct WhenView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) when: Box<dyn WhenErased<S> + Send + Sync>,
    pub(crate) view: Box<dyn quickapi_view::ViewTrait<S> + Send + Sync>,
}

#[derive(Default)]
pub struct WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) inner: Vec<WhenView<S>>,
}

impl<S> WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// add_when adds a view with a condition to the WhenViews.
    pub fn add_when<T, W, V>(&mut self, when: W, view: V)
    where
        T: Send + Sync + 'static,
        W: When<S, T> + Sync + Send + 'static,
        V: quickapi_view::ViewTrait<S> + Send + Sync + 'static,
    {
        self.inner.push(WhenView {
            when: Box::new(WhenBoxed {
                inner: when,
                _phantom: PhantomData,
            }),
            view: Box::new(view),
        });
    }
}

#[async_trait::async_trait]
#[allow(non_snake_case, missing_docs)]
impl<S, F> When<S, ()> for F
where
    S: Clone + Send + Sync + 'static,
    F: Fn(&mut Parts) -> Result<(), quickapi_view::Error> + Send + Sync + 'static,
{
    async fn when(&self, _parts: &mut Parts, _state: &S) -> Result<(), quickapi_view::Error> {
        (self)(_parts)
    }
}

macro_rules! impl_when_func {
    ([$($ty:ident),*], $last:ident) => {
        #[async_trait::async_trait]
        #[allow(non_snake_case, missing_docs)]
        impl<S, F, $($ty,)* $last> When<S, ($($ty,)* $last,)> for F
        where
            S: Clone + Send + Sync + 'static,
            $($ty: axum::extract::FromRequestParts<S> + Send + Sync + 'static, )*
            $last: axum::extract::FromRequestParts<S> + Send + Sync + 'static,
            F: Fn(&mut Parts, $($ty,)* $last) -> Result<(), quickapi_view::Error> + Send + Sync + 'static,
        {
            async fn when(&self, _parts: &mut Parts, _state: &S) -> Result<(), quickapi_view::Error> {
                $(
                    let $ty = $ty::from_request_parts(_parts, _state).await.map_err(|_| {
                        quickapi_view::Error::NoMatchWhen
                    })?;
                )*
                let $last = $last::from_request_parts(_parts, _state).await.map_err(|_| {
                    quickapi_view::Error::NoMatchWhen
                })?;

                (self)(_parts, $($ty,)* $last)
            }
        }
    }
}

quickapi_macro::all_the_tuples!(impl_when_func);

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use std::sync::Arc;

    fn hello(_parts: &mut Parts) -> Result<(), quickapi_view::Error> {
        Ok(())
    }

    #[tokio::test]
    async fn test_when_views() {
        let mut _when_views = WhenViews::<()>::default();
        _when_views.add_when(hello, ());
    }
}
