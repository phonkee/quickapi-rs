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

use super::{Limit, Page};
use crate::filter::SelectModelFilter;
use crate::filter::common::paginator::limit::{LimitChoices, LimitConstraint, LimitConstraintBox};
use crate::filter::common::paginator::params;
use axum::http::request::Parts;
use sea_orm::Select;

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct Paginator<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    pub(crate) page: Page,
    pub(crate) default_page: Page,
    pub(crate) limit: Limit,
    pub(crate) default_limit: Limit,
    pub(crate) limit_constraint: LimitConstraintBox,
    pub(crate) params: params::Params,
    _phantom: std::marker::PhantomData<(M, S)>,
}

/// Paginator is a filter that reads pagination from query, and applies to query and also to response.
impl<M, S> Paginator<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    /// parse_query extracts the page and limit parameters from a query string.
    pub fn parse_query(&mut self, query: impl AsRef<str>) -> Result<(), crate::filter::Error> {
        let (page, limit) = self.params.parse_query(query)?;

        self.limit = match limit {
            Some(l) => self.limit_constraint.limit(l, self.default_limit.clone())?,
            None => self.default_limit.clone(),
        };

        self.page = match page {
            Some(p) => p,
            None => self.default_page.clone(),
        };

        Ok(())
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

    /// with_per_page_accept sets the selected items per page.
    pub fn with_limit_choices<T, L>(self, choices: Vec<L>) -> Self
    where
        L: Into<Limit>,
    {
        self.with_limit_constraint(LimitChoices::from(choices))
    }

    /// with_limit_constraint sets the limit constraint for the paginator.
    pub fn with_limit_constraint<T: LimitConstraint>(mut self, constraint: T) -> Self {
        self.limit_constraint = Box::new(constraint);
        self
    }
}

#[async_trait::async_trait]
impl<M, S, X> SelectModelFilter<M, S, X> for Paginator<M, S>
where
    M: sea_orm::EntityTrait,
    S: Clone + Send + Sync + 'static,
    X: serde::Serialize + Clone + Send + Sync + 'static,
{
    async fn filter_select(
        &self,
        _parts: &mut Parts,
        _state: S,
        query: Select<M>,
    ) -> Result<Select<M>, crate::Error> {
        Ok(query)
    }

    /// is_last indicates that this filter is last in the chain of filters.
    fn is_last(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::common::paginator::Params;
    use sea_orm::entity::prelude::*;
    use serde::Serialize;
    use crate::filter::common::paginator::limit::LimitDefault;

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
    fn test_paginator_limit() {
        let paginator = Paginator::<Entity, ()>::default()
            .with_default_limit(20)
            .with_params_prefixed("custom");

        assert_eq!(paginator.limit, 20.into());
    }

    #[test]
    fn test_paginator_limit_constraint_choices() {
        let mut paginator = Paginator::<Entity, ()>::default()
            .with_params(Params::new("p", "l"))
            .with_default_limit(20)
            .with_limit_constraint(LimitChoices::from(vec![10, 20, 50]));

        paginator.parse_query("p=1&l=30").unwrap();

        assert_eq!(paginator.limit, 20.into());
        assert_eq!(paginator.page, 1.into());
    }

    #[test]
    fn test_paginator_limit_constraint_default() {
        let mut paginator = Paginator::<Entity, ()>::default()
            .with_params(Params::new("p", "l"))
            .with_default_limit(20)
            .with_limit_constraint(LimitDefault);

        paginator.parse_query("p=1&l=30").unwrap();

        assert_eq!(paginator.limit, 20.into());
        assert_eq!(paginator.page, 1.into());
    }

}
