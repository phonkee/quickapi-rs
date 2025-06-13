pub mod detail;
pub mod error;
pub mod handler;
pub mod list;
mod view;
pub mod when;
pub mod program;

pub use error::Error;
pub use view::ViewTrait;

pub use detail::DetailView;
pub use list::ListView;
