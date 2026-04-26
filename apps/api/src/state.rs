use common::Config;
use oxid_core::{OxidAuthService, OxidUseCase, create_service};
use server::errors::ServerError;
use std::sync::Arc;

use args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,

    pub auth: OxidAuthService,
    pub usecase: OxidUseCase,
}

pub async fn state(args: Arc<Args>) -> Result<AppState, ServerError> {
    let config = Config::from(args.as_ref().clone());

    let service = create_service(config).await.unwrap();

    Ok(AppState {
        args,
        auth: service.auth,
        usecase: service.usecase,
    })
}
