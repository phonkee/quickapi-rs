use crate::view::when::When;
use sea_orm::EntityTrait;
use std::marker::PhantomData;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone)]
pub struct WhenView<M, S> {
    pub when: Arc<dyn When<S, ()> + Send + Sync>,
    pub phantom_data: PhantomData<(M, S)>,
}

#[derive(Clone, Default)]
pub struct WhenViews<M, S>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    views: Vec<WhenView<M, S>>,
    phantom_data: PhantomData<(M, S)>,
}

impl<M, S> WhenViews<M, S>
where
    M: EntityTrait,
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            views: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    pub fn add_view(&mut self, view: WhenView<M, S>) {
        self.views.push(view);
    }

    pub fn views(&self) -> &[WhenView<M, S>] {
        &self.views
    }
}
