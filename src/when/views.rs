use crate::when::When;
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
    pub view: Arc<V>,
    pub phantom_data: PhantomData<(S, V)>,
}

impl<S, V> WhenView<S, V>
where
    S: Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(when: Arc<dyn When<S, ()> + Send + Sync>, view: V) -> Self {
        Self {
            when,
            view: view.into(),
            phantom_data: PhantomData,
        }
    }
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

    /// Adds a view with a condition to the WhenViews.
    pub fn add_view<T>(&mut self, _when: impl When<S, T> + Sync + Send, _view: V) {
        //self.views.push(WhenView::new(Arc::new(_when), view.into()));
    }
}
