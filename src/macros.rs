use axum::extract::{FromRequest, FromRequestParts};
use axum::handler::Handler;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use std::pin::Pin;

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

macro_rules! impl_handler {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        // #[allow(non_snake_case, unused_mut)]
        // impl<F, Fut, S, Res, M, $($ty,)* $last> Handler<(M, $($ty,)* $last,), S> for F
        // where
        //     F: FnOnce($($ty,)* $last,) -> Fut + Clone + Send + Sync + 'static,
        //     Fut: Future<Output = Res> + Send,
        //     S: Send + Sync + 'static,
        //     Res: IntoResponse,
        //     $( $ty: FromRequestParts<S> + Send, )*
        //     $last: FromRequest<S, M> + Send,
        // {
        //     type Future = Pin<Box<dyn Future<Output = Response> + Send>>;
        // 
        //     fn call(self, req: Request, state: S) -> Self::Future {
        //         let (mut parts, body) = req.into_parts();
        //         Box::pin(async move {
        //             $(
        //                 let $ty = match $ty::from_request_parts(&mut parts, &state).await {
        //                     Ok(value) => value,
        //                     Err(rejection) => return rejection.into_response(),
        //                 };
        //             )*
        // 
        //             let req = Request::from_parts(parts, body);
        // 
        //             let $last = match $last::from_request(req, &state).await {
        //                 Ok(value) => value,
        //                 Err(rejection) => return rejection.into_response(),
        //             };
        // 
        //             self($($ty,)* $last,).await.into_response()
        //         })
        //     }
        // }
    };
}

all_the_tuples!(impl_handler);
