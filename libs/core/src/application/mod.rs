use auth::{AuthService, FerrisKeyRepository};
use common::{Config, CoreError};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::infrastructure::postgres::error::map_sqlx_error;

pub mod member;
pub mod organization;
pub mod role;
pub mod user;

pub type OxidAuthService = AuthService<FerrisKeyRepository>;

#[derive(Clone)]
pub struct OxidUseCase {
    pool: PgPool,
}

impl OxidUseCase {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Clone)]
pub struct OxidService {
    pub auth: OxidAuthService,
    pub usecase: OxidUseCase,
}

impl OxidService {
    pub fn new(auth: OxidAuthService, usecase: OxidUseCase) -> Self {
        Self { auth, usecase }
    }
}

pub async fn create_service(config: Config) -> Result<OxidService, CoreError> {
    let auth_repo = FerrisKeyRepository::new(config.auth.issuer, None);
    let auth = AuthService::new(auth_repo);

    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.name,
    );
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .map_err(map_sqlx_error)?;

    Ok(OxidService::new(auth, OxidUseCase::new(pool)))
}
