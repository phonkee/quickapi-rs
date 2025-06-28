/*
 *  The MIT License (MIT)
 *
 *  Copyright (c) 2024-2025, Peter Vrba
 *
 *  Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the "Software"), to deal
 *  in the Software without restriction, including without limitation the rights
 *  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *  copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 *  all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 *  THE SOFTWARE.
 *
 */

use axum::http::request::Parts;
use dyn_clone::DynClone;
use std::marker::PhantomData;
use std::pin::Pin;

/// When trait for defining conditions that must be met before executing a view
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait When<S, T>: Send
where
    S: Clone + Send + Sync + 'static,
{
    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    async fn when(&self, _parts: &mut Parts, _state: &S) -> Result<(), crate::Error>;
}

#[allow(dead_code)]
pub(crate) trait WhenErased<S>: Send + DynClone
where
    S: Clone + Send + Sync + 'static,
{
    /// when is executed against the request and state and returns either error or success
    /// If success, the view is executed
    fn when<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send + 'a>>;
}

dyn_clone::clone_trait_object!(<S> WhenErased<S>);

pub struct WhenBoxed<F, S, T>
where
    F: When<S, T> + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    inner: F,
    _phantom: PhantomData<(S, T)>,
}

// Implement Clone for WhenBoxed
impl<F, S, T> Clone for WhenBoxed<F, S, T>
where
    F: When<S, T> + Send + Sync + Clone + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<F, S, T> WhenErased<S> for WhenBoxed<F, S, T>
where
    F: When<S, T> + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: Send + 'static,
{
    fn when<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send + 'a>> {
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

// Implement Clone for WhenView
impl<S> Clone for WhenView<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            when: dyn_clone::clone_box(&*self.when),
            view: dyn_clone::clone_box(&*self.view),
        }
    }
}

// Implement Clone for WhenViews
impl<S> Clone for WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.iter().map(|w| w.clone()).collect(),
        }
    }
}

impl<S> WhenView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn is_match<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send + 'a>> {
        self.when.when(parts, state)
    }
}

pub struct WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) inner: Vec<WhenView<S>>,
}

impl<S> Default for WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        WhenViews::new()
    }
}

impl<S> WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new instance of WhenViews.
    pub fn new() -> Self {
        WhenViews { inner: Vec::new() }
    }

    /// add_when adds a view with a condition to the WhenViews.
    pub fn add_when<T, W, V>(&mut self, when: W, view: V)
    where
        T: Send + Sync + 'static,
        W: When<S, T> + Clone + Sync + Send + 'static,
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

    /// count returns the number of views in WhenViews.
    pub fn count(&self) -> usize {
        self.inner.len()
    }

    /// is_empty checks if there are no views in WhenViews.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// get_view returns the first view that matches the condition.
    pub async fn get_view<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
    ) -> Result<&'a dyn quickapi_view::ViewTrait<S>, crate::Error> {
        for when_view in &self.inner {
            if when_view.is_match(parts, state).await.is_ok() {
                return Ok(when_view.view.as_ref());
            }
        }
        Err(crate::Error::NoMatch)
    }

    /// get_views returns list of all matching views.
    pub async fn get_views<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
    ) -> Result<Vec<&'a (dyn quickapi_view::ViewTrait<S> + Send + Sync)>, crate::Error> {
        let mut views = Vec::new();
        for when_view in &self.inner {
            if when_view.is_match(parts, state).await.is_ok() {
                views.push(when_view.view.as_ref());
            }
        }
        Ok(views)
    }
}

#[async_trait::async_trait]
#[allow(non_snake_case, missing_docs)]
impl<S, F, Fut> When<S, ()> for F
where
    S: Clone + Send + Sync + 'static,
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), crate::Error>> + Send + 'static,
{
    async fn when(&self, _parts: &mut Parts, _state: &S) -> Result<(), crate::Error> {
        (self)().await
    }
}

macro_rules! impl_when_func {
    ([$($ty:ident),*], $last:ident) => {
        #[async_trait::async_trait]
        #[allow(non_snake_case, missing_docs)]
        impl<S, F, Fut, $($ty,)* $last> When<S, ($($ty,)* $last,)> for F
        where
            S: Clone + Send + Sync + 'static,
            $($ty: axum::extract::FromRequestParts<S> + Send + Sync + 'static, )*
            $last: axum::extract::FromRequestParts<S> + Send + Sync + 'static,
            F: Fn($($ty,)* $last) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Result<(), crate::Error>> + Send + 'static,
        {
            async fn when(&self, _parts: &mut Parts, _state: &S) -> Result<(), crate::Error> {
                $(
                    let $ty = $ty::from_request_parts(_parts, _state).await.map_err(|_| {
                        crate::Error::NoMatch
                    })?;
                )*
                let $last = $last::from_request_parts(_parts, _state).await.map_err(|_| {
                    crate::Error::NoMatch
                })?;

                (self)($($ty,)* $last).await
            }
        }
    }
}

quickapi_macro::all_the_tuples!(impl_when_func);

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use axum::Extension;
    use axum::http::Extensions;

    // Example function to be used with WhenViews
    pub async fn hello(
        _u: axum::extract::OriginalUri,
        _x: axum::extract::State<()>,
    ) -> Result<(), crate::Error> {
        Ok(())
    }

    // Example function to be used with WhenViews
    pub async fn world() -> Result<(), crate::Error> {
        Ok(())
    }

    #[tokio::test]
    async fn test_when_views() {
        let mut _when_views = WhenViews::<()>::default();
        _when_views.add_when(hello, ());
        _when_views.add_when(world, ());
        _when_views.add_when(async move |_u: axum::extract::OriginalUri| Ok(()), ());

        // let mut ext = Extensions::new();
        // ext.insert()
    }
}
