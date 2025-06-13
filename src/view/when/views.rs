use crate::view::when::When;
use sea_orm::EntityTrait;
use std::marker::PhantomData;

#[allow(dead_code)]
pub struct WhenViews<M, O, S>
where
    M: EntityTrait,
    <M as EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    views: Vec<WhenView<M, O, S>>,
    phantom_data: PhantomData<(M, O, S)>,
}

#[allow(dead_code)]
pub struct WhenView<M, O, S> {
    pub when: Box<dyn When<S, ()> + Send + Sync>,
    pub phantom_data: PhantomData<(M, O, S)>,
}
