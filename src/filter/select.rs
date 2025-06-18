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

use crate::Error;
use crate::all_the_tuples;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sea_orm::Select;
use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;

#[async_trait::async_trait]
#[allow(missing_docs, non_snake_case)]
impl<F, M, S> super::SelectModelFilter<M, S, ()> for F
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Sync + Send + Clone + 'static,
    F: Fn(
            &mut Parts,
            S,
            Select<M>,
        ) -> Pin<Box<dyn Future<Output = Result<Select<M>, Error>> + Send>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    async fn filter_select(
        &self,
        parts: &mut Parts,
        state: S,
        query: Select<M>,
    ) -> Result<Select<M>, Error> {
        (self)(parts, state, query).await
    }
}

/// SelectFilters holds a vector of filters that can be applied to a Select query.
#[derive(Clone, Debug, Default)]
pub struct ModelFilters(pub Vec<Arc<dyn Any + Send + Sync>>);

/// Allows immutable access to the inner vector of filters.
impl Deref for ModelFilters {
    type Target = Vec<Arc<dyn Any + Send + Sync>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Allows mutable access to the inner vector of filters.
impl DerefMut for ModelFilters {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ModelFilters {
    /// push a new filter into the SelectFilters.
    pub fn push<M, S, T>(&mut self, filter: impl super::SelectModelFilter<M, S, T>)
    where
        M: sea_orm::EntityTrait + Send + Sync + 'static,
        S: Clone + Send + Sync + 'static,
    {
        self.0.push(Arc::new(filter));
    }
}

macro_rules! impl_filter_tuple {
    ([$($ty:ident),*], $last:ident) => {
        #[async_trait::async_trait]
        #[allow(missing_docs, non_snake_case)]
        impl<F, M, S, $($ty,)* $last> super::SelectModelFilter<M, S, ($($ty,)* $last,)> for F
        where
            M: sea_orm::EntityTrait + Send + Sync + 'static,
            S: Sync + Send + Clone + 'static,
            $(
                $ty: FromRequestParts<S> + Send + Sync + 'static,
            )*
            $last: FromRequestParts<S> + Send + Sync + 'static,
            F: Fn(&mut Parts, S, Select<M>, $($ty,)* $last) -> Result<Select<M>, Error> + Clone + Send + Sync + 'static,
        {
            async fn filter_select(
                &self,
                parts: &mut Parts,
                state: S,
                query: Select<M>,
            ) -> Result<Select<M>, Error> {

                $(
                let $ty = $ty::from_request_parts(parts, &state)
                    .await
                    .map_err(|_| Error::NoQueryFilterMatch)?;
                )*
                let $last = $last::from_request_parts(parts, &state)
                    .await
                    .map_err(|_| Error::NoQueryFilterMatch)?;

                (self)(parts, state, query, $($ty,)* $last,)
            }
        }
    };
}

all_the_tuples!(impl_filter_tuple);
