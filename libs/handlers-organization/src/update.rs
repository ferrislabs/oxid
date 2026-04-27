use axum::{Json, extract::State};
use handlers::{ApiError, AppState, Response};
use oxid_core::OrganizationId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrganizationPath, response::OrganizationResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub slug: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated", body = OrganizationResponse),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found"),
        (status = 409, description = "Slug already taken"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    OrganizationPath {
        organization_id: _organization_id,
    }: OrganizationPath,
    State(_state): State<AppState>,
    Json(_payload): Json<UpdateOrganizationRequest>,
) -> Result<Response<OrganizationResponse>, ApiError> {
    todo!("update organization")
}
