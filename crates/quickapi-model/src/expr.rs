/*
 *  The MIT License (MIT)
 *
 *  Copyright (c) 2024-2025, Peter Vrba
 *
 *  Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the "Software"), to deal
 *  in the Software without restriction, including without limitation the rights
 *  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *  copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 *  all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 *  THE SOFTWARE.
 *
 */
use sea_orm::sea_query::SimpleExpr;
use sea_orm::{ColumnTrait, ColumnType, Value};

macro_rules! error_value {
    ($col:expr, $value:expr, $ty:ident) => {
        crate::Error::ImproperlyConfigured(format!(
            "Failed to parse value '{}' as {} for column {:?}",
            $value,
            stringify!($ty),
            $col
        ))
    };
}

/// to_simple_expr converts a column and a value into a SimpleExpr.
pub fn to_simple_expr(col: impl ColumnTrait, value: String) -> Result<SimpleExpr, crate::Error> {
    let binding = col.def();
    let def = binding.get_column_type();
    Ok(match def {
        ColumnType::Char(_len) => SimpleExpr::Value(Value::String(Some(Box::new(value)))),
        ColumnType::String(_len) => SimpleExpr::Value(Value::String(Some(Box::new(value)))),
        ColumnType::Text => SimpleExpr::Value(Value::String(Some(Box::new(value)))),
        ColumnType::Blob => SimpleExpr::Value(Value::Bytes(Some(Box::new(value.into_bytes())))),
        ColumnType::Integer => {
            SimpleExpr::Value(Value::Int(Some(value.parse::<i32>().map_err(|_| {
                error_value!(col, value, i32)
            })?)))
        }
        ColumnType::BigInteger => {
            SimpleExpr::Value(Value::BigInt(Some(value.parse::<i64>().map_err(|_| {
                error_value!(col, value, i64)
            })?)))
        }
        _ => {
            todo!("finish all")
        }
    })
}
