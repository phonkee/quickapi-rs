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

use crate::RouterExt;
use crate::response::json::key::Key;
use crate::view::{delete, detail};
use axum::body::Body;
use axum::http::request::Parts;

/// ViewTrait defines the behavior of a view in the application.
#[async_trait::async_trait]
pub trait ViewTrait<S>: RouterExt<S> + Sync
where
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        parts: &mut Parts,
        state: S,
        body: Body,
    ) -> Result<crate::response::json::Response, crate::error::Error>;
}

/// ModelViewTrait defines the behavior of a model view in the application.
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait ModelViewTrait<M, S>: ViewTrait<S>
where
    M: sea_orm::entity::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    fn key(&self) -> Key;
}

/// View provides methods to create and manage views in the application.
pub struct View<S> {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) _marker: std::marker::PhantomData<S>,
}

/// View implements methods
impl<S> View<S> {
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
