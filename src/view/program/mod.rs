mod view;

pub use view::{ProgramView, new, new_with_method};

#[async_trait::async_trait]
pub trait ProgramViewTrait<S>
where
    S: Clone + Send + Sync + 'static,
{
}
