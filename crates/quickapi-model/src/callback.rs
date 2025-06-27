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
pub trait BeforeSave<M, S, T>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    /// filter_select is called to filter the select query.
    async fn before_save(
        &self,
        parts: &mut Parts,
        state: &S,
        model: M::Model,
    ) -> Result<M::Model, crate::Error>;
}

#[allow(dead_code)]
pub trait BeforeSaveErased<M, S>: Send + Sync + DynClone
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn callback_before_save<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        query: M::Model,
    ) -> Pin<Box<dyn Future<Output = Result<M::Model, crate::Error>> + Send + 'a>>;
}

dyn_clone::clone_trait_object!(<M, S> BeforeSaveErased<M, S>);

pub struct BeforeSaveBoxed<F, M, S, T>
where
    F: BeforeSave<M, S, T> + Send + Sync + 'static,
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    inner: F,
    _phantom: PhantomData<(M, S, T)>,
}

// Implement Clone for BeforeSaveBoxed
impl<F, M, S, T> Clone for BeforeSaveBoxed<F, M, S, T>
where
    F: BeforeSave<M, S, T> + Send + Sync + Clone + 'static,
    M: EntityTrait + Send + Sync + 'static,
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

impl<F, M, S, T> BeforeSaveErased<M, S> for BeforeSaveBoxed<F, M, S, T>
where
    F: BeforeSave<M, S, T> + Clone + Send + Sync + 'static,
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: Sync + Send + 'static,
{
    fn callback_before_save<'a>(
        &'a self,
        parts: &'a mut Parts,
        state: &'a S,
        model: M::Model,
    ) -> Pin<Box<dyn Future<Output = Result<M::Model, crate::Error>> + Send + 'a>> {
        Box::pin(self.inner.before_save(parts, state, model))
    }
}

/// BeforeSaveContainer is a container for multiple BeforeSave callbacks.
pub struct BeforeSaveContainer<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    pub(crate) inner: Vec<Box<dyn BeforeSaveErased<M, S> + Send + Sync>>,
}

impl<M, S> Default for BeforeSaveContainer<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// default creates a new BeforeSaveContainer with no callbacks.
    fn default() -> Self {
        Self::new()
    }
}

// Now you can derive Clone for BeforeSaveContainer
impl<M, S> Clone for BeforeSaveContainer<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
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
impl<M, S> BeforeSaveContainer<M, S>
where
    M: EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new BeforeSaveContainer.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// push adds a new BeforeSave callback to the container.
    pub fn push<F, T>(&mut self, f: F)
    where
        F: BeforeSave<M, S, T> + Clone + Send + Sync + 'static,
        T: Sync + Send + 'static,
    {
        let boxed = Box::new(BeforeSaveBoxed {
            inner: f,
            _phantom: PhantomData,
        });
        self.inner.push(boxed);
    }

    /// clear removes all BeforeSave callbacks from the container.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

macro_rules! impl_before_save_callback_tuple {
    ([$($ty:ident),*]) => {
        #[async_trait::async_trait]
        #[allow(missing_docs, non_snake_case, unused_variables)]
        impl<F, Fut, M, S, $($ty,)*> BeforeSave<M, S, ($($ty,)* )> for F
        where
            M: sea_orm::EntityTrait + Send + Sync + 'static,
            S: Sync + Send + Clone + 'static,
            F: Fn(M::Model, $($ty,)*) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = Result<M::Model, crate::Error>> + Send + 'static,
            $(
                $ty: axum::extract::FromRequestParts<S> + Send + 'static,
            )*
        {
            async fn before_save(
                &self,
                _parts: &mut Parts,
                _state: &S,
                model: M::Model,
            ) -> Result<M::Model, crate::Error> {
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
    pub async fn before_save_callback(model: Model) -> Result<Model, crate::Error> {
        // Here you can modify the model before saving it
        println!("Before saving model: {:?}", model);
        Ok(model)
    }

    #[test]
    fn test_before_save_trait() {
        let mut _container = BeforeSaveContainer::<Entity, ()>::new();
        _container.push(before_save_callback);
        _container.push(async move |m: Model| Ok(m));

        println!("Hello, BeforeSave trait!");
    }
}
