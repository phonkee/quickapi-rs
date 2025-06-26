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
use crate::view::{delete, detail};

/// Create a new instance of QuickApi with the provided database connection.
pub fn new<S>(db: sea_orm::DatabaseConnection) -> QuickApi<S> {
    QuickApi::<S> {
        db,
        _marker: std::marker::PhantomData,
    }
}

/// QuickApi is the main entry point for the QuickAPI framework, providing a database connection
#[derive(Debug)]
#[allow(dead_code)]
pub struct QuickApi<S> {
    /// db is the database connection used by the QuickAPI framework.
    pub(crate) db: sea_orm::DatabaseConnection,
    /// _marker is a marker type to ensure that QuickApi can be used with different state types.
    pub(crate) _marker: std::marker::PhantomData<S>,
}

/// QuickApi implements methods to create views in the application.
impl<S> QuickApi<S> {
    /// viewset returns object to create viewsets in the application.
    pub fn prefix(&self, path: impl AsRef<str>) -> crate::viewset::ViewSet<S> {
        crate::viewset::new(path.as_ref())
    }

    /// delete creates a new DeleteView for the specified path using the DELETE method.
    pub fn delete(&self) -> delete::View<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        delete::View::<S> {
            db: self.db.clone(),
            _marker: std::marker::PhantomData,
        }
    }

    /// detail creates a new DetailView for the specified path using the GET method.
    pub fn detail(&self) -> detail::View<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        detail::View {
            db: self.db.clone(),
            _marker: std::marker::PhantomData,
        }
    }

    /// list creates a new ListView for the specified path using the GET method.
    pub fn list(&self) -> crate::view::list::View<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        crate::view::list::View {
            db: self.db.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}
