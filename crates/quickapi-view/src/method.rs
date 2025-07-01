use axum::routing::MethodFilter;

/// Convert an axum::http::Method to a MethodFilter
pub fn as_method_filter(method: &axum::http::Method) -> Result<MethodFilter, crate::Error> {
    Ok(method.clone().try_into().map_err(|e| {
        crate::Error::InvalidMethod(format!(
            "Failed to convert method {} to MethodFilter: {}",
            method, e
        ))
    })?)
}
