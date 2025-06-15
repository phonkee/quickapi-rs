use crate::Error;
use axum::routing::MethodFilter;

/// Convert an axum::http::Method to a MethodFilter
pub(crate) fn as_method_filter(method: &axum::http::Method) -> Result<MethodFilter, Error> {
    Ok(method.clone().try_into().map_err(|e| {
        Error::InvalidMethod(format!(
            "Failed to convert method {} to MethodFilter: {}",
            method, e
        ))
    })?)
}
