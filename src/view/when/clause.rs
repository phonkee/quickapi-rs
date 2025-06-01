use crate::view::View;
use crate::view::when::When;
use std::marker::PhantomData;

/// Clause struct that represents a condition to be met before executing a view
pub struct Clause<S>
where
    S: Clone + Send + Sync + 'static,
    // V: View<S> + Send + Sync + 'static,
    // W: When<S, Future = F>,
{
    // when: W,
    phantom: PhantomData<S>,
}

pub struct Clauses<S>(Vec<Clause<S>>)
where
    S: Clone + Send + Sync + 'static;

impl<S> Default for Clauses<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Clauses(Vec::new())
    }
}
