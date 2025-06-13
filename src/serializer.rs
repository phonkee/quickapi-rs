#[allow(dead_code)]
pub trait SerializerJson<M, S>: Send + Sync + 'static
where
    M: sea_orm::entity::EntityTrait,
    S: serde::Serialize,
{
    /// serialize method serializes the model into a string.
    fn serialize_json(&self, model: &M::Model) -> Result<serde_json::Value, crate::error::Error>;
}

impl<M, S> SerializerJson<M, S> for M
where
    M: sea_orm::entity::EntityTrait,
    S: serde::Serialize + From<<M as sea_orm::EntityTrait>::Model>,
{
    fn serialize_json(&self, model: &M::Model) -> Result<serde_json::Value, crate::error::Error> {
        Ok(serde_json::to_value(Into::<S>::into(model.clone()))?)
    }
}

pub fn default_serializer<M, S>() -> impl SerializerJson<M, S>
where
    M: sea_orm::entity::EntityTrait,
    S: serde::Serialize + From<<M as sea_orm::EntityTrait>::Model>,
{
    M::default()
}

pub fn model_serializer<M>() -> impl SerializerJson<M, <M as sea_orm::EntityTrait>::Model>
where
    M: sea_orm::entity::EntityTrait,
    <M as sea_orm::EntityTrait>::Model: serde::Serialize + Clone + Send + Sync + 'static,
{
    default_serializer::<M, <M as sea_orm::EntityTrait>::Model>()
}
