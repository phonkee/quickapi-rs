use crate::view::when::When;
use std::marker::PhantomData;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone)]
pub struct WhenView<S, V>
where
    S: Clone + Send + Sync + 'static,
    V: Send + Sync + 'static + ?Sized,
{
    pub when: Arc<dyn When<S, ()> + Send + Sync>,
    pub phantom_data: PhantomData<(S, V)>,
}

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct WhenViews<S, V>
where
    S: Clone + Send + Sync + 'static,
    V: Send + Sync + 'static + ?Sized,
{
    views: Vec<WhenView<S, V>>,
    phantom_data: PhantomData<(S, V)>,
}

impl<S, V> WhenViews<S, V>
where
    S: Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            views: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    // pub fn add_view<T>(&mut self, when: impl When<S, T>, view: WhenView<S>) {
    //     self.views.push(view);
    // }
    //
    // pub fn views(&self) -> &[WhenView<S>] {
    //     &self.views
    // }
}
