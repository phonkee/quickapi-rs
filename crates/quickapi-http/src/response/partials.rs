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
use std::marker::PhantomData;

/// Parts struct that holds a collection of parts for a response.
#[derive(Clone, Debug)]
pub struct Partials<S> {
    parts: serde_json::Map<String, serde_json::Value>,
    phantom_data: PhantomData<S>,
}

/// Implementing Default for Parts<S>
impl<S> Default for Partials<S> {
    fn default() -> Self {
        Partials {
            parts: serde_json::Map::new(),
            phantom_data: PhantomData,
        }
    }
}

/// Implementing Parts methods
impl<S> Partials<S> {
    /// Inserts a part into the collection.
    pub fn insert(&mut self, key: String, value: serde_json::Value) {
        self.parts.insert(key, value);
    }

    /// update_map
    pub fn update_map(&self, other_map: &mut serde_json::Map<String, serde_json::Value>) {
        for (key, value) in &self.parts {
            other_map.insert(key.clone(), value.clone());
        }
    }
}

/// Convert Parts<S> to serde_json::Value
impl<S> From<Partials<S>> for serde_json::Value {
    fn from(s: Partials<S>) -> Self {
        serde_json::Value::Object(s.parts.clone())
    }
}
