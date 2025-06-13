#[derive(Clone, Debug)]
pub struct ModelSerializerJson<S>
where
    S: serde::Serialize + Clone + Send + Sync + 'static,
{
    _phantom: std::marker::PhantomData<S>,
}

impl<S> ModelSerializerJson<S>
where
    S: serde::Serialize + Clone + Send + Sync + 'static,
{
    /// Creates a new instance of ModelSerializerJson.
    pub fn new() -> Self {
        ModelSerializerJson {
            _phantom: std::marker::PhantomData,
        }
    }

    /// serializes the provided data into a JSON value.
    fn serialize_json(&self, data: impl Into<S>) -> Result<serde_json::Value, crate::error::Error> {
        let model: S = data.into();
        Ok(serde_json::to_value(model)?)
    }
}
