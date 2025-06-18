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

use crate::view::ViewTrait;
use axum::response::{IntoResponse, Response};
use std::marker::PhantomData;
use std::pin::Pin;

#[derive(Clone)]
pub(crate) struct Handler<S, V>(V, PhantomData<S>)
where
    V: ViewTrait<S> + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static;

impl<S, V> Handler<S, V>
where
    V: ViewTrait<S> + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new Handler instance with the given view and state.
    pub fn new(view: V) -> Self {
        Self(view, PhantomData)
    }
}

/// Implementing Handler for DetailView to handle requests.
impl<S, V> axum::handler::Handler<(), S> for Handler<S, V>
where
    V: ViewTrait<S> + Clone + Send + Sync,
    S: Clone + Send + Sync,
{
    // Only require Send, not Sync
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(self, _req: axum::extract::Request, _state: S) -> Self::Future {
        let (mut parts, body) = _req.into_parts();
        let state = _state.clone();

        Box::pin(async move {
            // prepare json response parts
            parts
                .extensions
                .insert(crate::response::parts::Parts::<S>::default());

            self.0
                .handle_view(&mut parts, state, body)
                .await
                .unwrap()
                .into_response()
        })
    }
}
