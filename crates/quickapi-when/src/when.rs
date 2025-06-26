use axum::http::request::Parts;
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
pub(crate) trait WhenErased<S>: Send
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
mod tests {
    use super::*;

    // Example function to be used with WhenViews
    pub async fn hello(
        _u: axum::extract::OriginalUri,
        _x: axum::extract::State<()>,
    ) -> Result<(), crate::Error> {
        Ok(())
    }

    // Example function to be used with WhenViews
    pub async fn world(_s: ()) -> Result<(), crate::Error> {
        Ok(())
    }

    #[tokio::test]
    async fn test_when_views() {
        let mut _when_views = WhenViews::<()>::default();
        _when_views.add_when(hello, ());
        _when_views.add_when(world, ());
        _when_views.add_when(async move |_u: axum::extract::OriginalUri| Ok(()), ());
    }
}
