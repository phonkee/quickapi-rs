use crate::view::filter::Filter;
use crate::view::list::ListView;
use axum::body::Body;
use sea_orm::Select;
use std::pin::Pin;
use std::sync::Arc;

pub trait When: Send + Sync {
    type Future: Future<Output = Result<(), crate::view::error::Error>> + Send + Sync + 'static;

    fn when(self, req: &mut axum::extract::Request) -> Self::Future;
}

impl When for () {
    type Future = Pin<
        Box<dyn Future<Output = Result<(), crate::view::error::Error>> + Send + Sync + 'static>,
    >;
    fn when(self, _req: &mut axum::extract::Request) -> Self::Future {
        Box::pin(async { Err(crate::view::error::Error::NotApplied {}) })
    }
}

#[derive(Clone)]
pub struct WhenView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    when: Arc<
        Box<
            dyn Filter<
                    S,
                    M,
                    Future = Pin<Box<dyn Future<Output = Result<Select<M>, ()>> + Send + 'static>>,
                >,
        >,
    >,
    view: ListView<M, S, O>,
}

impl<M, S, O> WhenView<M, S, O>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::entity::EntityTrait>::Model: Into<O>,
    S: Clone + Send + Sync + 'static,
    O: serde::Serialize + Clone + Send + Sync + 'static,
{
    pub fn with_serializer<Ser>(self) -> WhenView<M, S, Ser>
    where
        M: sea_orm::entity::EntityTrait,
        <M as sea_orm::entity::EntityTrait>::Model: Into<Ser>,
        S: Clone + Send + Sync + 'static,
        Ser: serde::Serialize + Clone + Send + Sync + 'static,
    {
        WhenView {
            when: self.when,
            view: self.view.with_serializer::<Ser>(),
        }
    }
}
