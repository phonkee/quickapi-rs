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
use crate::filter::SelectModelFilter;
use axum::http::request::Parts;
use sea_orm::Select;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Paginator<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    prefix: Option<String>,
    page: usize,
    per_page: usize,
    per_page_selected: Option<Vec<usize>>,
    _phantom: std::marker::PhantomData<(M, S)>,
}

impl<M, S> Default for Paginator<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            prefix: None,
            per_page_selected: None,
            page: 1,
            per_page: 10,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Paginator is a filter that reads pagination from query, and applies to query and also to response.
impl<M, S> Paginator<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new Paginator with the given prefix.
    pub fn new() -> Self {
        Self::default()
    }

    /// with_per_page sets the number of items per page.
    pub fn with_per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page;
        self
    }

    /// with_prefix sets the prefix for the paginator.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// with_per_page_selected sets the selected items per page.
    pub fn with_per_page_selected(mut self, selected: Vec<usize>) -> Self {
        self.per_page_selected = Some(selected);
        self
    }
}

#[async_trait::async_trait]
impl<M, S, X> SelectModelFilter<M, S, X> for Paginator<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
    X: serde::Serialize + Clone + Send + Sync + 'static,
{
    async fn filter_select(
        &self,
        _parts: &mut Parts,
        _state: S,
        query: Select<M>,
    ) -> Result<Select<M>, Error> {
        Ok(query)
    }
}
