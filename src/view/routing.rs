use axum::handler::Handler;
use axum::routing::{MethodFilter, MethodRouter, on};
use std::convert::Infallible;

pub fn get<H, S>(handler: H) -> MethodRouter<S, Infallible>
where
    H: Handler<(), S> + Clone + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    on::<H, (), S>(MethodFilter::GET, handler)
}
