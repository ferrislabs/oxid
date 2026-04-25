use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(title = "Oxid API", description = "API for Oxid", version = "0.1.0"))]
pub struct ApiDoc;
