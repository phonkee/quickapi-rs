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
use sea_orm::EntityTrait;
use std::marker::PhantomData;
use std::pin::Pin;

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait ModelCallback<E, S, T>
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    /// filter_select is called to filter the select query.
    async fn call(
        &self,
        parts: &mut Parts,
        state: &S,
        model: E::Model,
    ) -> Result<E::Model, crate::Error>;
}

#[allow(dead_code)]
pub trait ModelCallbackErased<E, S>: Send + Sync + DynClone
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn call<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: E::Model,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, crate::Error>> + Send + 'a>>;
}

dyn_clone::clone_trait_object!(<E, S> ModelCallbackErased<E, S>);

pub struct ModelCallbackBoxed<F, E, S, T>
where
    F: ModelCallback<E, S, T> + Send + Sync + 'static,
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    inner: F,
    _phantom: PhantomData<(E, S, T)>,
}

// Implement Clone for ModelCallbackBoxed
impl<F, E, S, T> Clone for ModelCallbackBoxed<F, E, S, T>
where
    F: ModelCallback<E, S, T> + Send + Sync + Clone + 'static,
    E: EntityTrait + Send + Sync + 'static,
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

impl<F, E, S, T> ModelCallbackErased<E, S> for ModelCallbackBoxed<F, E, S, T>
where
    F: ModelCallback<E, S, T> + Clone + Send + Sync + 'static,
    E: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: Sync + Send + 'static,
{
    fn call<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        model: E::Model,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, crate::Error>> + Send + 'a>> {
        Box::pin(self.inner.call(parts, state, model))
    }
}

/// ModelCallbacks is a container for multiple ModelCallback callbacks.
pub struct ModelCallbacks<E, S>
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    pub(crate) inner: Vec<Box<dyn ModelCallbackErased<E, S> + Send + Sync>>,
}

impl<E, S> Default for ModelCallbacks<E, S>
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// default creates a new ModelCallbackContainer with no callbacks.
    fn default() -> Self {
        Self::new()
    }
}

// Now you can derive Clone for ModelCallbackContainer
impl<E, S> Clone for ModelCallbacks<E, S>
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self
                .inner
                .iter()
                .map(|cb| dyn_clone::clone_box(&**cb))
                .collect(),
        }
    }
}

#[allow(dead_code)]
impl<E, S> ModelCallbacks<E, S>
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new ModelCallbackContainer.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// push adds a new ModelCallback callback to the container.
    pub fn push<F, T>(&mut self, f: F)
    where
        F: ModelCallback<E, S, T> + Clone + Send + Sync + 'static,
        T: Sync + Send + 'static,
    {
        let boxed = Box::new(ModelCallbackBoxed {
            inner: f,
            _phantom: PhantomData,
        });
        self.inner.push(boxed);
    }

    /// clear removes all ModelCallback callbacks from the container.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

/// Implement ModelCallbackErased for ModelCallbackContainer
impl<E, S> ModelCallbackErased<E, S> for ModelCallbacks<E, S>
where
    E: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn call<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        model: E::Model,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, crate::Error>> + Send + 'a>> {
        Box::pin(async move {
            let mut model = model;
            for cb in &self.inner {
                let result = cb.call(parts, state, model.clone()).await;

                match result {
                    Ok(updated_model) => {
                        model = updated_model;
                    }
                    Err(e) => {
                        match e {
                            crate::Error::NoMatch => {}
                            _ => {
                                // If any callback fails, return the error
                                return Err(e);
                            }
                        }
                    }
                };
            }
            Ok(model)
        })
    }
}

macro_rules! impl_before_save_callback_tuple {
    ([$($ty:ident),*]) => {
        #[async_trait::async_trait]
        #[allow(missing_docs, non_snake_case, unused_variables)]
        impl<F, Fut, E, S, $($ty,)*> ModelCallback<E, S, ($($ty,)* )> for F
        where
            E: sea_orm::EntityTrait + Send + Sync + 'static,
            S: Sync + Send + Clone + 'static,
            F: Fn(E::Model, $($ty,)*) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = Result<E::Model, crate::Error>> + Send + 'static,
            $(
                $ty: axum::extract::FromRequestParts<S> + Send + 'static,
            )*
        {
            async fn call(
                &self,
                _parts: &mut Parts,
                _state: &S,
                model: E::Model,
            ) -> Result<E::Model, crate::Error> {
                $(
                    let $ty = $ty::from_request_parts(_parts, _state).await.map_err(|_| {
                        crate::Error::NoMatch
                    })?;
                )*

                (self)(model, $($ty,)* ).await
            }
        }
    };
}

quickapi_macro::all_the_tuples_with_empty!(impl_before_save_callback_tuple);

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    // example callback function
    pub async fn before_save_callback(mut model: Model) -> Result<Model, crate::Error> {
        // Here you can modify the model before saving it
        model.id = 42;
        Ok(model)
    }

    #[tokio::test]
    async fn test_model_callbacks() {
        let mut _container = ModelCallbacks::<Entity, ()>::new();
        _container.push(before_save_callback);
        _container.push(async move |m: Model| Ok(m));

        // Create a dummy request to test the callback
        let req = axum::http::Request::builder()
            .method(axum::http::Method::POST)
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let (mut _parts, _body) = req.into_parts();

        let model_final = _container
            .call(&mut _parts, &(), Model { id: 1 })
            .await
            .unwrap();

        assert_eq!(
            model_final.id, 42,
            "Model ID should be modified by the callback"
        );
    }
}
