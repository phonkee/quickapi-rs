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

use axum::Router;
use axum::http::request::Parts;
use quickapi_http::Response;
use quickapi_view::{Error, RouterExt, ViewTrait};

/// Prefix is a struct that contains multiple views under a common path prefix.
#[allow(dead_code)]
pub struct Prefix<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) path: String,
    pub(crate) views: Vec<Box<dyn RouterExt<S> + Send + Sync>>,
}

/// Prefix implements Clone to allow cloning of the struct.
impl<S> Clone for Prefix<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Prefix {
            path: self.path.clone(),
            views: self
                .views
                .iter()
                .map(|v| dyn_clone::clone_box(&**v))
                .collect(),
        }
    }
}

/// Prefix implements methods to create and manage views under a common path prefix.
impl<S> Prefix<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// new creates a new instance of Prefix with the specified path.
    pub fn new(path: impl AsRef<str>) -> Self {
        Prefix {
            path: path.as_ref().to_string(),
            views: Vec::new(),
        }
    }

    /// with_view adds a view to the Prefix.
    #[allow(unused_mut)]
    pub fn with_view<V>(mut self, view: V) -> Self
    where
        V: RouterExt<S> + Send + Sync + 'static,
    {
        self.views.push(Box::new(view));
        self
    }
}

/// Prefix implements the ViewTrait for handling requests under the specified path prefix.
#[async_trait::async_trait]
impl<S> quickapi_view::ViewTrait<S> for Prefix<S>
where
    S: Clone + Send + Sync + 'static,
{
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: &S,
        _body: &bytes::Bytes,
    ) -> Result<Response, Error> {
        todo!()
    }
    /// get_when_views returns an empty vector for Prefix, as it does not have when views.
    async fn get_when_views<'a>(
        &'a self,
        _parts: &'a mut Parts,
        _state: &'a S,
    ) -> Result<Vec<&'a (dyn ViewTrait<S> + Send + Sync)>, Error> {
        Ok(vec![])
    }

    // TODO: no we does not have fallback view by default
    fn has_fallback(&self) -> bool {
        // Prefix does not have a fallback view by default
        false
    }
}

/// Prefix implements the RouterExt trait for registering routers with a path prefix.
impl<S> quickapi_view::RouterExt<S> for Prefix<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn register_router_with_prefix(
        &self,
        _router: Router<S>,
        _prefix: &str,
    ) -> Result<Router<S>, Error> {
        let mut router = _router;
        let prefixed_path = format!("{}{}", _prefix, self.path);
        let _span = tracing::debug_span!("API prefix", prefix = &prefixed_path);
        let _x = _span.enter();

        for view in &self.views {
            // Register each view with the router using the prefix
            let updated_router =
                view.register_router_with_prefix(router.clone(), &prefixed_path)?;
            router = updated_router;
        }

        Ok(router)
    }
}
