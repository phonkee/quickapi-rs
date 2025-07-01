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
use super::{DEFAULT_LIMIT, Limit, Page};
use crate::common::paginator::limit::LimitConstraint;
use crate::common::paginator::params;
use crate::select::SelectFilter;
use async_trait::async_trait;
use axum::http::request::Parts;

#[derive(Clone, Debug, Default)]
pub struct Paginator<E, S>
where
    E: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    pub(crate) page: Page,
    pub(crate) default_page: Page,
    pub(crate) limit: Limit,
    pub(crate) default_limit: Limit,
    pub(crate) limit_constraint: LimitConstraint,
    pub(crate) params: params::Params,
    _phantom: std::marker::PhantomData<(E, S)>,
}

/// Paginator is a filter that reads pagination from query, and applies to query and also to response.
impl<E, S> Paginator<E, S>
where
    E: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// parse_query extracts the page and limit parameters from a query string.
    pub fn parse_query(&self, query: impl AsRef<str>) -> Result<(Page, Limit), crate::Error> {
        let (page, limit) = self.params.parse_query(query)?;

        let limit = match limit {
            Some(l) => self.limit_constraint.limit(l, self.default_limit.clone())?,
            None => self.default_limit.clone(),
        };

        let page = match page {
            Some(p) => p,
            None => self.default_page.clone(),
        };

        Ok((page, limit))
    }

    /// with_default_limit sets the default limit for the paginator.
    pub fn with_default_limit(mut self, limit: impl Into<Limit>) -> Self {
        self.default_limit = limit.into();
        self
    }

    /// with_params_prefixed sets the parameter names for the paginator with a prefix.
    pub fn with_params_prefixed(self, prefix: impl Into<String>) -> Self {
        self.with_params(params::Params::new_prefixed(prefix.into()))
    }

    /// with_params sets the parameter names for the paginator.
    pub fn with_params(mut self, names: params::Params) -> Self {
        self.params = names;
        self
    }

    /// with_limit_constraint sets the limit constraint for the paginator.
    pub fn with_limit_constraint(mut self, constraint: impl Into<LimitConstraint>) -> Self {
        self.limit_constraint = constraint.into();
        self
    }
}

#[async_trait::async_trait]
impl<E, S> SelectFilter<E, S, ()> for Paginator<E, S>
where
    E: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    async fn filter_select(
        &self,
        _parts: &mut Parts,
        _state: &S,
        query: sea_orm::Select<E>,
    ) -> Result<sea_orm::Select<E>, crate::Error> {
        let query_str = _parts.uri.query().unwrap_or_default();

        let (_page, _limit) = self.parse_query(query_str)?;
        // let offset = (page.0 - 1) * limit.0;
        // TODO: add to parts information for response pagination
        // Ok(query.limit(limit.0).offset(offset).into())

        Ok(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::paginator::Params;
    use sea_orm::entity::prelude::*;
    use serde::Serialize;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[test]
    fn test_paginator() {
        let _paginator = Paginator::<Entity, ()>::default();
        // assert_eq!(paginator.per_page, 20);
    }

    #[test]
    fn test_paginator_default_limit() {
        let paginator = Paginator::<Entity, ()>::default().with_params_prefixed("custom");

        assert_eq!(paginator.default_limit, DEFAULT_LIMIT.into());
    }

    #[test]
    fn test_paginator_limit_constraint_choices() {
        let mut paginator = Paginator::<Entity, ()>::default()
            .with_params(Params::new("p", "l"))
            .with_default_limit(20)
            .with_limit_constraint(vec![10, 20, 50]);

        let (page, limit) = paginator.parse_query("p=1&l=30").unwrap();

        assert_eq!(limit, 20.into());
        assert_eq!(page, 1.into());
    }

    #[test]
    fn test_paginator_limit_constraint_default() {
        let mut paginator = Paginator::<Entity, ()>::default()
            .with_params(Params::new("p", "l"))
            .with_default_limit(20)
            .with_limit_constraint(LimitConstraint::Default);

        let (page, limit) = paginator.parse_query("p=1&l=30").unwrap();

        assert_eq!(limit, 20.into());
        assert_eq!(page, 1.into());
    }
}
