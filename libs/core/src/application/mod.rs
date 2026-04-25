use auth::{AuthService, FerrisKeyRepository};
use common::{Config, CoreError};
use sqlx::postgres::PgPoolOptions;

use crate::application::user::UserUseCase;
use crate::infrastructure::postgres::error::map_sqlx_error;

pub mod user;

pub type OxidAuthService = AuthService<FerrisKeyRepository>;

#[derive(Clone)]
pub struct OxidService {
    pub auth: OxidAuthService,
    pub users: UserUseCase,
}

impl OxidService {
    pub fn new(auth: OxidAuthService, users: UserUseCase) -> Self {
        Self { auth, users }
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

    let users = UserUseCase::new(pool);

    Ok(OxidService::new(auth, users))
}
