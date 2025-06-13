#[derive(Clone, Debug)]
pub struct JsonResponse {
    pub data: serde_json::Value,
    pub status: axum::http::StatusCode,
    pub headers: axum::http::HeaderMap,
}

/// Default implementation for JsonResponse
impl Default for JsonResponse {
    fn default() -> Self {
        JsonResponse {
            data: serde_json::Value::Null,
            status: axum::http::StatusCode::OK,
            headers: axum::http::HeaderMap::new(),
        }
    }
}

/// Implementing IntoResponse for JsonResponse to convert it into an axum response
impl axum::response::IntoResponse for JsonResponse {
    fn into_response(self) -> axum::response::Response {
        let mut response = axum::response::Response::new(self.data.to_string().into());
        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;
        response
    }
}
