use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use axum_extra::routing::TypedPath;
use handlers::{AppState, auth::auth_middleware};

use crate::paths::{CurrentUserOrganizationsPath, OrganizationPath, OrganizationsPath};

pub mod create;
pub mod get_one;
pub mod list_mine;
pub mod paths;
pub mod response;
pub mod soft_delete;
pub mod update;

pub const TAG: &str = "organizations";

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route(OrganizationsPath::PATH, post(create::handler))
        .route(
            OrganizationPath::PATH,
            get(get_one::handler)
                .patch(update::handler)
                .delete(soft_delete::handler),
        )
        .route(CurrentUserOrganizationsPath::PATH, get(list_mine::handler))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
