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
use std::fmt::Debug;
use std::str::FromStr;

const DEFAULT_LIMIT: usize = 10;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub struct Limit(usize);

impl Default for Limit {
    fn default() -> Self {
        Limit(DEFAULT_LIMIT)
    }
}

impl From<usize> for Limit {
    fn from(value: usize) -> Self {
        Limit(value)
    }
}

impl FromStr for Limit {
    type Err = crate::filter::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s
            .parse::<usize>()
            .map_err(|_| crate::filter::Error::InvalidQueryParameter(format!("limit: {s}")))?;
        if value == 0 {
            return Err(crate::filter::Error::InvalidQueryParameter(
                "limit: 0".to_string(),
            ));
        }
        Ok(Self(value))
    }
}

/// LimitConstraintBox is a type alias for a boxed LimitConstraint trait object.
pub type LimitConstraintBox = Box<dyn LimitConstraint + Send + Sync>;

impl Default for LimitConstraintBox {
    fn default() -> Self {
        Box::new(())
    }
}

/// LimitConstraint is a trait that defines how to handle limits in pagination.
pub trait LimitConstraint: Send + Sync + 'static + Debug {
    fn clone_box(&self) -> Box<dyn LimitConstraint + Send + Sync>;
    fn limit(&self, value: Limit, default: Limit) -> Result<Limit, crate::filter::Error>;
}

// Implement Clone for Box<dyn LimitConstraint>
impl Clone for Box<dyn LimitConstraint + Send + Sync> {
    fn clone(&self) -> Box<dyn LimitConstraint + Send + Sync> {
        self.clone_box()
    }
}

/// LimitConstraint for tuple
impl LimitConstraint for () {
    fn clone_box(&self) -> Box<dyn LimitConstraint + Send + Sync> {
        Box::new(())
    }
    fn limit(&self, value: Limit, _default: Limit) -> Result<Limit, crate::filter::Error> {
        Ok(value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct LimitChoices(pub Vec<Limit>);

impl<T> From<Vec<T>> for LimitChoices
where
    T: Into<Limit>,
{
    fn from(choices: Vec<T>) -> Self {
        LimitChoices(choices.into_iter().map(Into::into).collect())
    }
}

/// Implements a limit constraint that restricts the limit to a set of predefined choices.
impl LimitConstraint for LimitChoices {
    fn clone_box(&self) -> Box<dyn LimitConstraint + Send + Sync> {
        Box::new(self.clone())
    }
    fn limit(&self, value: Limit, default: Limit) -> Result<Limit, crate::filter::Error> {
        if self.0.is_empty() {
            return Ok(value);
        }

        Ok(if self.0.contains(&value) {
            value
        } else {
            default
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct LimitStatic(pub Limit);

impl LimitConstraint for LimitStatic {
    fn clone_box(&self) -> Box<dyn LimitConstraint + Send + Sync> {
        Box::new(self.clone())
    }
    fn limit(&self, _value: Limit, _default: Limit) -> Result<Limit, crate::filter::Error> {
        Ok(self.0.clone())
    }
}

#[derive(Clone, Debug, Default)]
pub struct LimitDefault;

impl LimitConstraint for LimitDefault {
    fn clone_box(&self) -> Box<dyn LimitConstraint + Send + Sync> {
        Box::new(self.clone())
    }
    fn limit(&self, _value: Limit, default: Limit) -> Result<Limit, crate::filter::Error> {
        Ok(default)
    }
}
