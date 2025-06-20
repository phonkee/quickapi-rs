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
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sea_orm::Select;

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait SelectModel<M, S, T>
where
    M: sea_orm::EntityTrait + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    T: 'static,
{
    /// filter_select is called to filter the select query.
    async fn filter_select(
        &self,
        parts: &mut Parts,
        state: &S,
        query: Select<M>,
    ) -> Result<Select<M>, crate::filter::Error>;
}

macro_rules! impl_select_tuples {
    ([$($ty:ident),*], $last:ident) => {
        #[async_trait::async_trait]
        #[allow(missing_docs, non_snake_case)]
        impl<F, M, S, $($ty,)* $last> SelectModel<M, S, ($($ty,)* $last,)> for F
        where
            M: sea_orm::EntityTrait + Send + Sync + 'static,
            S: Sync + Send + Clone + 'static,
            F: Fn(sea_orm::Select<M>, $($ty,)* $last) -> Result<sea_orm::Select<M>, crate::filter::Error> + Sync,
            $(
                $ty: axum::extract::FromRequestParts<S> + Send + 'static,
            )*
            $last: FromRequestParts<S> + Send + 'static,
        {
            async fn filter_select(
                &self,
                parts: &mut Parts,
                state: &S,
                query: sea_orm::Select<M>,
            ) -> Result<sea_orm::Select<M>, crate::filter::Error> {
                $(
                    let $ty = $ty::from_request_parts(parts, state).await.map_err(|_| {
                        crate::filter::Error::NoMatch
                    })?;
                )*
                let $last = $last::from_request_parts(parts, state).await.map_err(|_| {
                    crate::filter::Error::NoMatch
                })?;

                (self)(query, $($ty,)* $last)
            }
        }
    };
}

all_the_tuples!(impl_select_tuples);
