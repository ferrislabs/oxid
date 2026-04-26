use axum::{Router, extract::Request, routing::get};
use tower_http::trace::TraceLayer;
use tracing::info_span;

use crate::errors::ApiError;

pub fn internal_router() -> Result<Router, ApiError> {
    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request| {
        let uri: String = request.uri().to_string();
        info_span!("internal_http_request", method = ?request.method(), uri)
    });

    let router = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .layer(trace_layer);

    Ok(router)
}

async fn health() -> &'static str {
    "ok"
}

async fn metrics() -> &'static str {
    ""
}
