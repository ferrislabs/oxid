use axum::{Json, body::Body, response::IntoResponse};
use http::StatusCode;
use serde::Serialize;

pub enum Response<T: Serialize + PartialEq> {
    OK(T),
    Created(T),
    NoContent,
    Accepted(T),
}

impl<T: Serialize + PartialEq> IntoResponse for Response<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            Response::OK(data) => (StatusCode::OK, Json(data)).into_response(),
            Response::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Response::Accepted(data) => (StatusCode::ACCEPTED, Json(data)).into_response(),
            Response::NoContent => (StatusCode::NO_CONTENT, Body::empty()).into_response(),
        }
    }
}
