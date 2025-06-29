/*
 *  The MIT License (MIT)
 *
 *  Copyright (c) 2024-2025, Peter Vrba
 *
 *  Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the "Software"), to deal
 *  in the Software without restriction, including without limitation the rights
 *  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *  copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 *  all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 *  THE SOFTWARE.
 *
 */

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
    #[allow(dead_code)]
    pub fn serialize_json(
        &self,
        data: impl Into<S>,
    ) -> Result<serde_json::Value, crate::error::Error> {
        let model: S = data.into();
        Ok(serde_json::to_value(model)?)
    }
}

#[derive(Clone, Debug)]
pub struct ModelDeserializerJson<S>
where
    S: for<'a> serde::Deserialize<'a> + Clone + Send + Sync + 'static,
{
    _phantom: std::marker::PhantomData<S>,
}

impl<S> ModelDeserializerJson<S>
where
    S: for<'a> serde::Deserialize<'a> + Clone + Send + Sync + 'static,
{
    /// Creates a new instance of ModelDeserializerJson.
    pub fn new() -> Self {
        ModelDeserializerJson {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Deserializes the provided JSON value into the specified type.
    pub fn deserialize_json<M>(&self, _data: &bytes::Bytes) -> Result<M::Model, crate::error::Error>
    where
        M: sea_orm::EntityTrait,
    // M::Model: From<S>,
    {
        let _intermediate: S = serde_json::from_slice(&_data)?;
        todo!()
        // Ok(_intermediate.into())
    }
}
