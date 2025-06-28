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
use crate::Error;
use crate::view::delete::DeleteView;
use crate::view::{detail::DetailView, list::ListView};
use axum::http::Method;
use quickapi_lookup::Lookup;
use sea_orm::{EntityTrait, Iden, Iterable};

/// Create a new instance of QuickApi with the provided database connection.
pub fn new<S>(db: sea_orm::DatabaseConnection) -> QuickApi<S> {
    QuickApi::<S> {
        db,
        _marker: std::marker::PhantomData,
    }
}

/// QuickApi is the main entry point for the QuickAPI framework, providing a database connection
#[derive(Debug)]
#[allow(dead_code)]
pub struct QuickApi<S> {
    /// db is the database connection used by the QuickAPI framework.
    pub(crate) db: sea_orm::DatabaseConnection,
    /// _marker is a marker type to ensure that QuickApi can be used with different state types.
    pub(crate) _marker: std::marker::PhantomData<S>,
}

/// QuickApi implements methods to create views in the application.
impl<S> QuickApi<S> {
    /// create creates a new CreateView instance with a specified path and method.
    pub fn create<M>(
        &self,
        path_method: impl Into<CreatePathMethod>,
    ) -> Result<crate::view::create::CreateView<M, S, M::Model>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model:
            serde::Serialize + for<'a> serde::Deserialize<'a> + Into<M::Model>,
    {
        let _path_method = path_method.into();
        crate::view::create::CreateView::<M, S, M::Model>::new(
            self.db.clone(),
            _path_method.path,
            _path_method.method,
        )
    }

    /// delete creates a new DeleteView instance with a specified path and method.
    pub fn delete<M>(
        &self,
        path_method: impl Into<DeletePathMethod>,
    ) -> Result<DeleteView<M, S>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
    {
        let path_method = path_method.into();

        // Get the first primary key column name as a string
        let primary_key = M::PrimaryKey::iter()
            .next()
            .ok_or(Error::ImproperlyConfigured(
                "No primary key found for entity".to_string(),
            ))?
            .to_string();

        Ok(DeleteView::new(
            self.db.clone(),
            path_method.path,
            path_method.method,
            primary_key,
        ))
    }

    /// detail
    pub fn detail<M>(
        &self,
        path_method: impl Into<DetailPathMethod>,
        lookup: impl Lookup<M, S> + 'static,
    ) -> Result<DetailView<M, S, M::Model>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        let pm = path_method.into();

        Ok(DetailView::<M, S, M::Model>::new(
            self.db.clone(),
            pm.path,
            pm.method,
            lookup,
        ))
    }

    /// list creates a new ListView instance with a specified path and method.
    pub fn list<M>(
        &self,
        path_method: impl Into<ListPathMethod>,
    ) -> Result<ListView<M, S, M::Model>, Error>
    where
        M: EntityTrait,
        S: Clone + Send + Sync + 'static,
        <M as EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
    {
        let pm = path_method.into();
        Ok(ListView::<M, S, M::Model>::new(
            self.db.clone(),
            pm.path,
            pm.method,
        ))
    }

    /// viewset returns object to create viewsets in the application.
    pub fn prefix(&self, path: impl AsRef<str>) -> crate::view::prefix::Prefix<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        crate::view::prefix::Prefix::new(path.as_ref())
    }
}

macro_rules! impl_into_path_method {
    ($ty:ident, $method:path) => {
        #[derive(Clone, Debug)]
        #[allow(dead_code, missing_docs)]
        pub struct $ty {
            path: String,
            method: axum::http::Method,
        }
        impl From<String> for $ty {
            fn from(path: String) -> Self {
                Self {
                    path,
                    method: $method,
                }
            }
        }
        impl From<&str> for $ty {
            fn from(path: &str) -> Self {
                Self {
                    path: path.to_owned(),
                    method: $method,
                }
            }
        }
        impl From<(String, axum::http::Method)> for $ty {
            fn from((path, method): (String, axum::http::Method)) -> Self {
                Self { path, method }
            }
        }
        impl From<(&str, axum::http::Method)> for $ty {
            fn from((path, method): (&str, axum::http::Method)) -> Self {
                Self {
                    path: path.to_owned(),
                    method,
                }
            }
        }
    };
}

impl_into_path_method!(DetailPathMethod, Method::GET);
impl_into_path_method!(CreatePathMethod, Method::POST);
impl_into_path_method!(ListPathMethod, Method::GET);
impl_into_path_method!(DeletePathMethod, Method::DELETE);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_path_method() {
        let x: CreatePathMethod = "test".to_string().into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::POST);

        let x: CreatePathMethod = "test".into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::POST);

        let x: CreatePathMethod = ("test".to_owned(), Method::PUT).into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::PUT);

        let x: CreatePathMethod = ("test", Method::POST).into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::POST);
    }

    #[test]
    fn test_delete_path_method() {
        let x: DeletePathMethod = "test".to_string().into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::DELETE);

        let x: DeletePathMethod = "test".into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::DELETE);

        let x: DeletePathMethod = ("test".to_owned(), Method::PATCH).into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::PATCH);

        let x: DeletePathMethod = ("test", Method::DELETE).into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::DELETE);
    }

    #[test]
    fn test_detail_path_method() {
        let x: DetailPathMethod = "test".to_string().into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::GET);

        let x: DetailPathMethod = "test".into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::GET);

        let x: DetailPathMethod = ("test".to_owned(), Method::POST).into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::POST);

        let x: DetailPathMethod = ("test", Method::GET).into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::GET);
    }

    #[test]
    fn test_list_path_method() {
        let x: ListPathMethod = "test".to_string().into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::GET);

        let x: ListPathMethod = "test".into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::GET);

        let x: ListPathMethod = ("test".to_owned(), Method::HEAD).into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::HEAD);

        let x: ListPathMethod = ("test", Method::GET).into();
        assert_eq!(x.path, "test");
        assert_eq!(x.method, Method::GET);
    }
}
