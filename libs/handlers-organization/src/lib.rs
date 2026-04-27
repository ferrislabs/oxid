use axum::{Router, routing::get};
use axum_extra::routing::TypedPath;
use handlers::AppState;

use crate::paths::{OrganizationPath, OrganizationsPath};

pub mod create;
pub mod get_one;
pub mod list;
pub mod paths;
pub mod response;
pub mod soft_delete;
pub mod update;

pub const TAG: &str = "organizations";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            OrganizationsPath::PATH,
            get(list::handler).post(create::handler),
        )
        .route(
            OrganizationPath::PATH,
            get(get_one::handler)
                .patch(update::handler)
                .delete(soft_delete::handler),
        )
}
