use crate::view::View;
use crate::view::when::When;
use std::marker::PhantomData;

/// Clause struct that represents a condition to be met before executing a view
pub struct Clause<F, S, V, W>
where
    F: Future<Output = Result<(), crate::view::error::Error>>,
    S: Clone + Send + Sync + 'static,
    V: View<S> + Send + Sync + 'static,
    W: When<S, Future = F>,
{
    when: W,
    phantom: PhantomData<(F, S, V, W)>,
}

pub type Clauses<F, S, V, W> = Vec<Clause<F, S, V, W>>;
