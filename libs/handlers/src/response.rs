use axum::{Json, body::Body, response::IntoResponse};
use http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct DataEnvelope<T: Serialize> {
    pub data: T,
}

impl<T: Serialize> DataEnvelope<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

pub enum Response<T: Serialize + PartialEq> {
    OK(T),
    Created(T),
    NoContent,
    Accepted(T),
}

impl<T: Serialize + PartialEq> IntoResponse for Response<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            Response::OK(data) => (StatusCode::OK, Json(DataEnvelope::new(data))).into_response(),
            Response::Created(data) => {
                (StatusCode::CREATED, Json(DataEnvelope::new(data))).into_response()
            }
            Response::Accepted(data) => {
                (StatusCode::ACCEPTED, Json(DataEnvelope::new(data))).into_response()
            }
            Response::NoContent => (StatusCode::NO_CONTENT, Body::empty()).into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Sample {
        id: u32,
        name: String,
    }

    fn sample() -> Sample {
        Sample {
            id: 42,
            name: "oxid".to_owned(),
        }
    }

    async fn parse<T: Serialize + PartialEq>(
        response: Response<T>,
    ) -> (StatusCode, Option<serde_json::Value>) {
        let response = response.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json = if bytes.is_empty() {
            None
        } else {
            Some(serde_json::from_slice(&bytes).unwrap())
        };
        (status, json)
    }

    #[test]
    fn data_envelope_serializes_with_data_key() {
        let envelope = DataEnvelope::new(sample());
        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["data"]["id"], 42);
        assert_eq!(json["data"]["name"], "oxid");
        assert_eq!(json.as_object().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn ok_wraps_payload_in_data_envelope() {
        let (status, json) = parse(Response::OK(sample())).await;
        assert_eq!(status, StatusCode::OK);
        let json = json.expect("OK must have a body");
        assert_eq!(json["data"]["id"], 42);
        assert_eq!(json["data"]["name"], "oxid");
    }

    #[tokio::test]
    async fn created_wraps_payload_in_data_envelope() {
        let (status, json) = parse(Response::Created(sample())).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json.unwrap()["data"]["id"], 42);
    }

    #[tokio::test]
    async fn accepted_wraps_payload_in_data_envelope() {
        let (status, json) = parse(Response::Accepted(sample())).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(json.unwrap()["data"]["id"], 42);
    }

    #[tokio::test]
    async fn no_content_has_empty_body() {
        let (status, json) = parse(Response::<Sample>::NoContent).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(json.is_none());
    }

    #[tokio::test]
    async fn ok_with_vec_wraps_array_under_data_key() {
        let (_, json) = parse(Response::OK(vec![sample(), sample()])).await;
        let data = &json.unwrap()["data"];
        assert!(data.is_array());
        assert_eq!(data.as_array().unwrap().len(), 2);
    }
}
