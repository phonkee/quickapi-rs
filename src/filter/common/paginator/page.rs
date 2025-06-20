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
use std::str::FromStr;

// Page type
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub struct Page(usize);

impl Default for Page {
    fn default() -> Self {
        Self(1)
    }
}

impl From<usize> for Page {
    fn from(value: usize) -> Self {
        if value == 0 { Self(1) } else { Self(value) }
    }
}

// implement parse from string
impl FromStr for Page {
    type Err = crate::filter::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s
            .parse::<usize>()
            .map_err(|_| crate::filter::Error::InvalidQueryParameter("page".into()))?;
        if value == 0 {
            return Err(crate::filter::Error::InvalidQueryParameter(
                "page".to_string(),
            ));
        }
        Ok(Self(value))
    }
}
