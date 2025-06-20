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
use crate::filter::common::paginator::{Limit, Page};

const DEFAULT_PAGE: &str = "page";
const DEFAULT_PER_PAGE: &str = "limit";

#[derive(Debug, Clone)]
pub struct Params {
    pub page: String,
    pub limit: String,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE.to_owned(),
            limit: DEFAULT_PER_PAGE.to_owned(),
        }
    }
}

impl Params {
    /// new_prefixed creates a new instance of Names with the given prefix.
    pub fn new_prefixed(prefix: impl AsRef<str>) -> Self {
        let prefix = prefix.as_ref().trim_end_matches('_');
        let mut result = Self::default();
        if !prefix.is_empty() {
            result.page = format!("{prefix}_{}", result.page);
            result.limit = format!("{prefix}_{}", result.limit);
        }
        result
    }

    /// parse_query extracts the page and limit parameters from a query string.
    pub fn parse_query(
        &self,
        uri: &axum::http::Uri,
    ) -> Result<(Option<Page>, Option<Limit>), crate::filter::Error> {
        let Some(path_query) = uri.path_and_query() else {
            return Ok((None, None));
        };

        // now get query parameters
        let query = path_query.query().unwrap_or("");

        let mut page: Option<Page> = None;
        let mut limit: Option<Limit> = None;

        // now parse query parameters
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            match key.as_ref() {
                _ if key == self.page => {
                    page = Some(value.parse::<Page>()?);
                }
                _ if key == self.limit => {
                    limit = Some(value.parse::<Limit>()?);
                }
                _ => continue,
            };
        }

        Ok((page, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_names() {
        let names = Params::default();
        assert_eq!(names.page, DEFAULT_PAGE);
        assert_eq!(names.limit, DEFAULT_PER_PAGE);
    }

    #[test]
    fn test_new_prefixed() {
        let names = Params::new_prefixed("custom");
        assert_eq!(names.page, "custom_page");
        assert_eq!(names.limit, "custom_limit");
    }

    #[test]
    fn test_new_prefixed_empty() {
        let names = Params::new_prefixed("");
        assert_eq!(names.page, DEFAULT_PAGE);
        assert_eq!(names.limit, DEFAULT_PER_PAGE);
    }

    #[test]
    fn test_parse_query() {
        let params = Params::default();
        let uri = axum::http::Uri::from_static("/items?page=2&limit=10");
        let (page, limit) = params.parse_query(&uri).unwrap();
        assert_eq!(page, Some(2.into()));
        assert_eq!(limit, Some(10.into()));
    }
}
