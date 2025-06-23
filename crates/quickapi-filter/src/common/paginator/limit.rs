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

/// DEFAULT_LIMIT is the default limit for pagination.
pub const DEFAULT_LIMIT: usize = 10;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub struct Limit(usize);

impl Default for Limit {
    fn default() -> Self {
        Limit(DEFAULT_LIMIT)
    }
}

// implement parse from string
impl FromStr for Limit {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s
            .parse::<usize>()
            .map_err(|_| crate::Error::InvalidQueryParameter("limit".into()))?;
        if value == 0 {
            return Err(crate::Error::InvalidQueryParameter("limit".to_string()));
        }
        Ok(Self(value))
    }
}

/// Convert from usize to Limit
impl From<usize> for Limit {
    fn from(value: usize) -> Self {
        Limit(value)
    }
}

/// Convert from numeric types to Limit
macro_rules! impl_from_limit {
    ($($t:ty),+) => {
        $(
            impl From<$t> for Limit {
                fn from(value: $t) -> Self {
                    Limit(value as usize)
                }
            }
        )+
    };
}

// Implement From for various numeric types to Limit
impl_from_limit!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

#[derive(Clone, Debug, Default)]
pub enum LimitConstraint {
    #[default]
    Default,
    Any,
    Choices(Vec<Limit>),
    Static(Limit),
}

/// Convert from limit Vec to Choices
impl<T> From<Vec<T>> for LimitConstraint
where
    T: Into<Limit>,
{
    fn from(value: Vec<T>) -> Self {
        LimitConstraint::Choices(value.into_iter().map(Into::into).collect())
    }
}

/// Convert from a single limit value to Static
impl<T> From<T> for LimitConstraint
where
    T: Into<Limit>,
{
    fn from(value: T) -> Self {
        LimitConstraint::Static(value.into())
    }
}

/// Implementation of LimitConstraint for a tuple of limits
impl LimitConstraint {
    // limit returns the limit based on the constraint.
    pub fn limit(
        &self,
        value: impl Into<Limit>,
        default: impl Into<Limit>,
    ) -> Result<Limit, crate::Error> {
        Ok(match self {
            LimitConstraint::Default => default.into(),
            LimitConstraint::Any => value.into(),
            LimitConstraint::Choices(choices) => {
                let value = value.into();
                if choices.contains(&value) {
                    value
                } else {
                    default.into()
                }
            }
            LimitConstraint::Static(static_limit) => static_limit.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_constraints_default() {
        let constraint = LimitConstraint::Default;

        let result = constraint.limit(10, 20).unwrap();
        assert_eq!(result, Limit(20));
    }

    #[test]
    fn test_limit_constraints_any() {
        let constraint = LimitConstraint::Any;

        let result = constraint.limit(10, 20).unwrap();
        assert_eq!(result, Limit(10));
    }

    #[test]
    fn test_limit_constraints_choices() {
        let constraint = LimitConstraint::Choices(vec![Limit(10), Limit(20), Limit(30)]);

        let result = constraint.limit(22, 50).unwrap();
        assert_eq!(result, Limit(50));

        let result = constraint.limit(25, 50).unwrap();
        assert_eq!(result, Limit(50)); // Default since 25 is not in choices
    }

    #[test]
    fn test_limit_constraints_static() {
        let constraint = LimitConstraint::Static(Limit(15));

        let result = constraint.limit(10, 20).unwrap();
        assert_eq!(result, Limit(15));
    }
}
