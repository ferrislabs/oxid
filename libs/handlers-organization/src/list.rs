use axum::extract::State;
use handlers::{ApiError, AppState, Response};

use crate::{paths::OrganizationsPath, response::OrganizationResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations",
    operation_id = "listOrganizations",
    tag = super::TAG,
    responses(
        (status = 200, description = "List organizations the caller belongs to", body = [OrganizationResponse]),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: OrganizationsPath,
    State(_state): State<AppState>,
) -> Result<Response<Vec<OrganizationResponse>>, ApiError> {
    let t: Vec<OrganizationResponse> = Vec::new();

    Ok(Response::OK(t))
}
