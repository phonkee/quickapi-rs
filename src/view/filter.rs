use axum::handler::Handler;
use sea_orm::{EntityTrait, Select};

pub trait Filter<M>
where
    M: sea_orm::EntityTrait,
{
    fn filter(
        &self,
        req: &mut axum::extract::Request,
        sel: sea_orm::Select<M>,
    ) -> Result<sea_orm::Select<M>, crate::view::error::Error>;
}

impl<M, F> Filter<M> for F
where
    M: sea_orm::EntityTrait,
    F: Fn(
        &mut axum::extract::Request,
        sea_orm::Select<M>,
    ) -> Result<sea_orm::Select<M>, crate::view::error::Error>,
{
    fn filter(
        &self,
        _req: &mut axum::extract::Request,
        sel: sea_orm::Select<M>,
    ) -> Result<sea_orm::Select<M>, crate::view::error::Error> {
        (self)(_req, sel)
    }
}
