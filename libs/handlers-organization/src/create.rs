use axum::{Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrganizationsPath, response::OrganizationResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub slug: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    operation_id = "createOrganization",
    tag = super::TAG,
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created", body = inline(DataEnvelope<OrganizationResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Slug already taken"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: OrganizationsPath,
    State(_state): State<AppState>,
    Json(_payload): Json<CreateOrganizationRequest>,
) -> Result<Response<OrganizationResponse>, ApiError> {
    todo!("create organization: build CreateOrganizationCommand and call usecase")
}
