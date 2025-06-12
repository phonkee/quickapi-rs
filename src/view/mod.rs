pub mod detail;
pub mod error;
pub mod filter;
pub mod handler;
pub mod list;
mod view;
pub mod when;

use crate::router::RouterExt;
use crate::view::detail::DetailView;
use axum::extract::Request;
use axum::http::request::Parts;
pub use error::Error;
use std::pin::Pin;

pub use view::ViewTrait;
