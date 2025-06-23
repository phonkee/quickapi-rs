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
use crate::view::program::ProgramViewTrait;
use axum::http::Method;
use std::marker::PhantomData;

/// new creates a new ProgramView with the specified path and defaults to the GET method.
pub fn new<S>(path: impl Into<String>, _what: impl ProgramViewTrait<S>) -> ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    new_with_method(path, Method::GET, _what)
}

/// new_with_method creates a new ProgramView with the specified path and method.
pub fn new_with_method<S>(
    path: impl Into<String>,
    method: Method,
    _what: impl ProgramViewTrait<S>,
) -> ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    ProgramView::<S> {
        path: path.into(),
        method,
        _marker: PhantomData,
        // when: WhenViews::<S>::new(),
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ProgramView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) path: String,
    pub(crate) method: Method,
    _marker: PhantomData<S>,
    // pub(crate) when: WhenViews<S>,
}

/// ProgramViewTrait is a trait that defines the behavior of a program view.
impl<S> ProgramViewTrait<S> for ProgramView<S> where S: Clone + Send + Sync + 'static {}
