use crate::view::ViewTrait;
use crate::when::When;
use std::marker::PhantomData;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone)]
pub struct WhenView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub when: Arc<dyn When<S, ()> + Send + Sync>,
    pub view: Arc<dyn ViewTrait<S> + Send + Sync + 'static>,
}

impl<S> WhenView<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(
        when: Arc<dyn When<S, ()> + Send + Sync>,
        view: Arc<dyn ViewTrait<S> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            when,
            view: view.into(),
        }
    }
}

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    views: Vec<WhenView<S>>,
    phantom_data: PhantomData<(S,)>,
}

impl<S> WhenViews<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            views: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    /// Adds a view with a condition to the WhenViews.
    pub fn add_view<T>(
        &mut self,
        _when: impl When<S, T> + Sync + Send,
        _view: Arc<dyn ViewTrait<S> + Send + Sync + 'static>,
    ) {
        //self.views.push(WhenView::new(Arc::new(_when), view.into()));
    }
}
