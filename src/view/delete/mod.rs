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
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::{Iden, Iterable};
use std::marker::PhantomData;
pub use view::DeleteView;

/// View to create detail views in the application.
pub struct View<S> {
    pub(crate) db: DatabaseConnection,
    pub(crate) _marker: PhantomData<S>,
}

/// View implements methods
impl<S> View<S> {
    pub fn new<M>(&self, path: impl AsRef<str>) -> Result<DeleteView<M, S>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
    {
        self.new_with_method(path.as_ref(), Method::DELETE)
    }

    /// new_with_method function that creates a new DetailView instance with a specified HTTP method
    /// new_with_method creates a new DeleteView instance with a specified method.
    pub fn new_with_method<M>(
        &self,
        path: impl Into<String>,
        method: Method,
    ) -> Result<DeleteView<M, S>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
    {
        // Get the first primary key column name as a string
        let primary_key = M::PrimaryKey::iter()
            .next()
            .ok_or(Error::ImproperlyConfigured(
                "No primary key found for entity".to_string(),
            ))?
            .to_string();

        Ok(DeleteView::new(
            self.db.clone(),
            &path.into(),
            method,
            primary_key,
        ))
    }
}
