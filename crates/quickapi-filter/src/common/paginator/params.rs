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
use crate::common::paginator::{Limit, Page};

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
    /// new creates a new instance of Names with the given page and limit.
    pub fn new(page: impl Into<String>, limit: impl Into<String>) -> Self {
        Self {
            page: page.into(),
            limit: limit.into(),
        }
    }

    /// new_prefixed creates a new instance of Names with the given prefix.
    pub fn new_prefixed(prefix: impl AsRef<str>) -> Self {
        let mut result = Self::default();
        let prefix = prefix.as_ref().trim_end_matches('_');
        if prefix.is_empty() || prefix == "" {
            return result; // Return default if prefix is empty
        }
        result.page = format!("{prefix}_{}", result.page);
        result.limit = format!("{prefix}_{}", result.limit);
        result
    }

    /// parse_query extracts the page and limit parameters from a query string.
    pub fn parse_query(
        &self,
        query: impl AsRef<str>,
    ) -> Result<(Option<Page>, Option<Limit>), crate::Error> {
        // prepare variables for page and limit
        let mut page = None;
        let mut limit = None;

        // // now parse query parameters
        for (key, value) in url::form_urlencoded::parse(query.as_ref().as_bytes()) {
            match key.as_ref() {
                _ if key == self.page => {
                    // TODO: match error
                    page = Some(
                        value
                            .parse::<Page>()
                            .map_err(|_| crate::Error::InvalidQueryParameter(self.page.clone()))?,
                    );
                }
                _ if key == self.limit => {
                    // TODO: match error
                    limit =
                        Some(value.parse::<Limit>().map_err(|_| {
                            crate::Error::InvalidQueryParameter(self.limit.clone())
                        })?);
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
        let (page, limit) = params.parse_query("page=2&limit=10").unwrap();
        assert_eq!(page, Some(2.into()));
        assert_eq!(limit, Some(10.into()));

        let result = params.parse_query("page=asdf&limit=10");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Invalid query parameter: page"
        );

        let result = params.parse_query("page=1&limit=fff");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Invalid query parameter: limit"
        );
    }

    #[test]
    fn test_parse_query_zero_page() {
        let params = Params::new_prefixed("custom");
        let result = params.parse_query("custom_page=0&custom_limit=10");
        assert_eq!(
            result.err().unwrap().to_string(),
            "Invalid query parameter: custom_page"
        );
    }

    #[test]
    fn test_empty_values() {
        let params = Params::default();
        let (page, limit) = params.parse_query("").unwrap();
        assert_eq!(page, None);
        assert_eq!(limit, None);
    }
}
