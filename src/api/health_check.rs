use hyper::{Response, StatusCode, Body};

use crate::helpers::response::{ApiResponse, ApiResponseBody};


pub fn health_check() -> ApiResponse {
    let ok_body = Body::from(ApiResponseBody::new("Ok".to_string(), Some("".to_string())));

    Response::builder()
        .status(StatusCode::OK).body(ok_body)
}