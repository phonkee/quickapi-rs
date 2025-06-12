pub mod detail;
pub mod error;
pub mod filter;
pub mod handler;
pub mod list;
mod view;
pub mod when;

use crate::router::RouterExt;
use axum::extract::Request;
use axum::http::request::Parts;
pub use error::Error;
use std::pin::Pin;

pub use view::ViewTrait;

pub use detail::DetailView;
pub use list::ListView;
