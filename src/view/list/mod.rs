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

pub mod view;

use crate::Error;
use axum::http::Method;
use std::marker::PhantomData;
pub use view::ListView;

#[async_trait::async_trait]
pub trait ListViewTrait<M, S>: quickapi_view::ViewTrait<S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
}

/// View to create detail views in the application.
pub struct View<S> {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) _marker: PhantomData<S>,
}

/// View implements methods
impl<S> View<S> {
    pub fn new<M>(&self, path: impl AsRef<str>) -> Result<ListView<M, S, M::Model>, Error>
    where
        M: sea_orm::EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as sea_orm::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        self.new_with_method(path, Method::GET)
    }

    /// new_with_method function that creates a new DetailView instance with a specified HTTP method
    pub fn new_with_method<M>(
        &self,
        path: impl AsRef<str>,
        method: Method,
    ) -> Result<ListView<M, S, M::Model>, Error>
    where
        M: sea_orm::EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as sea_orm::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        Ok(ListView::<M, S, M::Model>::new(
            self.db.clone(),
            path.as_ref(),
            method,
        ))
    }
}
