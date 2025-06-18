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
    /// view returns object to create views in the application.
    /// view is single endpoint in api.
    pub fn view(&self) -> crate::view::View<S> {
        crate::view::View {
            db: self.db.clone(),
            _marker: std::marker::PhantomData,
        }
    }

    /// viewset returns object to create viewsets in the application.
    pub fn viewset(&self, path: impl AsRef<str>) -> crate::viewset::ViewSet<S> {
        crate::viewset::new(path.as_ref())
    }
}
