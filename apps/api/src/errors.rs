use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    // 400
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    // 401
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Token not found")]
    TokenNotFound,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    // 403
    #[error("Forbidden")]
    Forbidden,

    #[error("Insufficient permissions: required scope `{0}`")]
    InsufficientScope(String),

    // 404
    #[error("Resource not found")]
    NotFound,

    #[error("{0} with id `{1}` not found")]
    ResourceNotFound(&'static str, String),

    // 409
    #[error("Conflict: {0}")]
    Conflict(String),

    // 422
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    // 429
    #[error("Too many requests")]
    TooManyRequests,

    // 500
    #[error("Internal server error")]
    Internal,

    #[error("Database error")]
    Database,

    #[error("External service error: {0}")]
    ExternalService(String),
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ApiError {
    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized
            | Self::TokenNotFound
            | Self::TokenExpired
            | Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::Forbidden | Self::InsufficientScope(_) => StatusCode::FORBIDDEN,
            Self::NotFound | Self::ResourceNotFound(_, _) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal | Self::Database | Self::ExternalService(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::TokenNotFound => "TOKEN_NOT_FOUND",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::Forbidden => "FORBIDDEN",
            Self::InsufficientScope(_) => "INSUFFICIENT_SCOPE",
            Self::NotFound => "NOT_FOUND",
            Self::ResourceNotFound(_, _) => "RESOURCE_NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::UnprocessableEntity(_) => "UNPROCESSABLE_ENTITY",
            Self::TooManyRequests => "TOO_MANY_REQUESTS",
            Self::Internal => "INTERNAL_SERVER_ERROR",
            Self::Database => "DATABASE_ERROR",
            Self::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorBody {
            code: self.code(),
            message: self.to_string(),
            details: None,
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn parse_response(err: ApiError) -> (StatusCode, serde_json::Value) {
        let response = err.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        (status, json)
    }

    macro_rules! assert_error {
        ($err:expr, $status:expr, $code:expr) => {{
            let (status, json) = parse_response($err).await;
            assert_eq!(status, $status);
            assert_eq!(json["code"], $code);
            assert!(json["message"].is_string());
            assert!(json.get("details").is_none());
        }};
    }

    #[tokio::test]
    async fn test_400_bad_request() {
        assert_error!(
            ApiError::BadRequest("invalid field".into()),
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST"
        );
    }

    #[tokio::test]
    async fn test_400_validation() {
        assert_error!(
            ApiError::Validation("email is required".into()),
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR"
        );
    }

    #[tokio::test]
    async fn test_401_unauthorized() {
        assert_error!(ApiError::Unauthorized, StatusCode::UNAUTHORIZED, "UNAUTHORIZED");
    }

    #[tokio::test]
    async fn test_401_token_not_found() {
        assert_error!(ApiError::TokenNotFound, StatusCode::UNAUTHORIZED, "TOKEN_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_401_token_expired() {
        assert_error!(ApiError::TokenExpired, StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED");
    }

    #[tokio::test]
    async fn test_401_invalid_token() {
        assert_error!(ApiError::InvalidToken, StatusCode::UNAUTHORIZED, "INVALID_TOKEN");
    }

    #[tokio::test]
    async fn test_403_forbidden() {
        assert_error!(ApiError::Forbidden, StatusCode::FORBIDDEN, "FORBIDDEN");
    }

    #[tokio::test]
    async fn test_403_insufficient_scope() {
        assert_error!(
            ApiError::InsufficientScope("admin:write".into()),
            StatusCode::FORBIDDEN,
            "INSUFFICIENT_SCOPE"
        );
    }

    #[tokio::test]
    async fn test_404_not_found() {
        assert_error!(ApiError::NotFound, StatusCode::NOT_FOUND, "NOT_FOUND");
    }

    #[tokio::test]
    async fn test_404_resource_not_found() {
        let (status, json) =
            parse_response(ApiError::ResourceNotFound("Customer", "42".into())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["code"], "RESOURCE_NOT_FOUND");
        assert!(json["message"].as_str().unwrap().contains("Customer"));
        assert!(json["message"].as_str().unwrap().contains("42"));
    }

    #[tokio::test]
    async fn test_409_conflict() {
        assert_error!(
            ApiError::Conflict("email already exists".into()),
            StatusCode::CONFLICT,
            "CONFLICT"
        );
    }

    #[tokio::test]
    async fn test_422_unprocessable_entity() {
        assert_error!(
            ApiError::UnprocessableEntity("invalid state transition".into()),
            StatusCode::UNPROCESSABLE_ENTITY,
            "UNPROCESSABLE_ENTITY"
        );
    }

    #[tokio::test]
    async fn test_429_too_many_requests() {
        assert_error!(
            ApiError::TooManyRequests,
            StatusCode::TOO_MANY_REQUESTS,
            "TOO_MANY_REQUESTS"
        );
    }

    #[tokio::test]
    async fn test_500_internal() {
        assert_error!(ApiError::Internal, StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR");
    }

    #[tokio::test]
    async fn test_500_database() {
        assert_error!(ApiError::Database, StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR");
    }

    #[tokio::test]
    async fn test_500_external_service() {
        assert_error!(
            ApiError::ExternalService("payment gateway timeout".into()),
            StatusCode::INTERNAL_SERVER_ERROR,
            "EXTERNAL_SERVICE_ERROR"
        );
    }

    #[tokio::test]
    async fn test_response_has_no_details_by_default() {
        let (_, json) = parse_response(ApiError::Internal).await;
        assert!(json.get("details").is_none());
    }
}
