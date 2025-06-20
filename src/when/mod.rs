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

mod views;
pub mod when;

pub use views::{WhenView, WhenViews};

/// When trait for defining conditions that must be met before executing a view
#[async_trait::async_trait]
pub trait When<S, T>: Send
where
    S: Clone + Send,
{
    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    async fn case_when(
        self,
        _parts: axum::http::request::Parts,
        _state: S,
    ) -> Result<(), crate::Error>;
}

#[async_trait::async_trait]
pub trait WhenBoxed<S>: Send
where
    S: Clone + Send,
{
    /// when is executed against the request and state
    /// when it succeeds, the view is executed
    async fn case_when(
        self,
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<(), crate::Error>;
}

/// CloneNoWhen trait for cloning objects without the Whens
pub trait CloneNoWhen {
    /// Clone the object without the When trait
    fn clone_without_when(&self) -> Self;
}
