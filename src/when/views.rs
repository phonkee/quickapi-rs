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
use crate::when::When;
use std::marker::PhantomData;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone)]
pub struct WhenView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub when: Arc<dyn When<S, ()> + Send + Sync>,
    pub view: Arc<dyn ViewTrait<S> + Send + Sync + 'static>,
}

impl<S> WhenView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(
        when: Arc<dyn When<S, ()> + Send + Sync>,
        view: Arc<dyn ViewTrait<S> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            when,
            view: view.into(),
        }
    }
}

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    views: Vec<WhenView<S>>,
    phantom_data: PhantomData<(S,)>,
}

impl<S> WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            views: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    /// Adds a view with a condition to the WhenViews.
    pub fn add_view<T>(
        &mut self,
        _when: impl When<S, T> + Sync + Send,
        _view: Arc<dyn ViewTrait<S> + Send + Sync + 'static>,
    ) {
        //self.views.push(WhenView::new(Arc::new(_when), view.into()));
    }
}
