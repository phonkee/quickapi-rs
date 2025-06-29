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
use axum::http::request::Parts;
use dyn_clone::DynClone;

/// ViewTrait defines the behavior of a view in the application.
#[async_trait::async_trait]
pub trait ViewTrait<S>: crate::RouterExt<S> + Sync + DynClone
where
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        parts: &mut Parts,
        state: S,
        body: bytes::Bytes,
    ) -> Result<quickapi_http::response::Response, Error>;

    /// get_when_views returns a vector of views that should be executed based on the request and state.
    async fn get_when_views<'a>(
        &'a self,
        _parts: &'a mut Parts,
        _state: &'a S,
    ) -> Result<Vec<&(dyn ViewTrait<S> + Send + Sync)>, Error>;

    /// has_fallback returns true if the view has a fallback view.
    fn has_fallback(&self) -> bool;

    /// run runs top level view logic.
    /// This is the entry point for the view and is only implemented in trait. all other trait methods must be implemented to work properly
    async fn run(
        &self,
        _parts: &mut Parts,
        _state: S,
        _body: &bytes::Bytes,
    ) -> Result<quickapi_http::response::Response, Error> {
        let mut _original_parts = _parts.clone();

        // check if we have when views
        // list all views
        let when_views = self.get_when_views(&mut _original_parts, &_state).await?;

        // when we have when views, we try to run them
        if !when_views.is_empty() {
            // prepare parts to be reused
            let mut _view_parts = _parts.clone();

            // iterate over all when views and try to run them
            for when_view in when_views {
                // how to clone body here?
                match when_view
                    .run(&mut _view_parts, _state.clone(), &_body)
                    .await
                {
                    Ok(_response) => {
                        //if we have a response, we return it
                        return Ok(_response);
                    }
                    Err(e) => match e {
                        Error::NoMatch => continue,
                        _ => {
                            // if we have an error, we return it
                            return Err(e);
                        }
                    },
                };
            }

            // now that we are here and tried everything, check if we have a fallback view
            if !self.has_fallback() {
                // Not found nothing
                return Ok(quickapi_http::response::Response {
                    data: serde_json::Value::Null,
                    status: axum::http::StatusCode::NOT_FOUND,
                    ..Default::default()
                });
            }
        }

        // now let's run the actual view logic
        match self.handle_view(_parts, _state, _body.clone()).await {
            Ok(response) => {
                // if we have a response, we return it
                Ok(response.with_header(axum::http::header::CONTENT_TYPE, "application/json"))
            }
            Err(e) => {
                // if we have an error, we return it
                Err(e)
            }
        }
    }
}

#[async_trait::async_trait]
impl<S> ViewTrait<S> for ()
where
    S: Clone + Send + Sync + 'static,
{
    /// handle_view runs the view logic.
    async fn handle_view(
        &self,
        _parts: &mut Parts,
        _state: S,
        _body: bytes::Bytes,
    ) -> Result<quickapi_http::response::Response, Error> {
        Ok(quickapi_http::response::Response::default())
    }

    /// get_when_views returns a list of views that match given request parts and state.
    async fn get_when_views<'a>(
        &'a self,
        _parts: &'a mut Parts,
        _state: &'a S,
    ) -> Result<Vec<&'a (dyn ViewTrait<S> + Send + Sync)>, Error> {
        Ok(vec![])
    }

    /// has_fallback returns true if the view has a fallback view (no match in whens or no whens).
    fn has_fallback(&self) -> bool {
        true
    }
}

#[async_trait::async_trait]
pub trait ViewWrapResultTrait<S>: ViewTrait<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// wrap_result_key is used to wrap the result in a JSON object with a specific key.
    fn wrap_result_key(self, key: impl Into<quickapi_http::response::key::Key>) -> Self;

    /// no_wrap_result_key disables wrapping the result in a JSON object with a specific key.
    fn no_wrap_result_key(self) -> Self;
}
