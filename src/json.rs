use std::ops::{Deref, DerefMut};

use axum::{
    http::{
        header::{self, IntoHeaderName},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
};
use bytes::{BufMut, BytesMut};
use serde::{ser::SerializeStruct, Serialize};

#[derive(Debug, Clone)]
pub struct JsonResult<T: Serialize, E: Serialize> {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Result<T, E>,
}

impl<T, E> Deref for JsonResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    type Target = Result<T, E>;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl<T, E> DerefMut for JsonResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

impl<T, E> From<Result<T, E>> for JsonResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn from(body: Result<T, E>) -> Self {
        match body {
            Ok(_) => Self {
                status: StatusCode::OK,
                headers: HeaderMap::new(),
                body,
            },
            Err(_) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                headers: HeaderMap::new(),
                body,
            },
        }
    }
}

impl<T, E> IntoResponse for JsonResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_json::to_writer(&mut buf, &self) {
            Ok(()) => (
                self.status,
                {
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                    );
                    headers.extend(self.headers);
                    headers
                },
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}

impl<T, E> Serialize for JsonResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("JsonResult", 2)?;
        state.serialize_field("status", &self.status.as_u16())?;
        match &self.body {
            Ok(body) => state.serialize_field("data", body)?,
            Err(err) => state.serialize_field("error", err)?,
        }
        state.end()
    }
}

impl<T, E> JsonResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

pub struct JsonBuilder<T: Serialize, E: Serialize>(JsonResult<T, E>);

impl<T, E> JsonBuilder<T, E>
where
    T: Serialize,
    E: Serialize,
{
    pub fn new(body: Result<T, E>) -> Self {
        Self(JsonResult::from(body))
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.0.status = status;
        self
    }

    pub fn header<K>(mut self, name: K, value: HeaderValue) -> Self
    where
        K: IntoHeaderName,
    {
        self.0.headers.insert(name, value);
        self
    }

    pub fn build(self) -> JsonResult<T, E> {
        self.0
    }
}
