use auth::{AuthService, FerrisKeyRepository};
use common::{Config, CoreError};
use rate_limit::{Quota, RateLimitService, RedisRateLimiter};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::infrastructure::postgres::error::map_sqlx_error;

pub mod member;
pub mod organization;
pub mod role;
pub mod user;

pub type OxidAuthService = AuthService<FerrisKeyRepository>;
pub type OxidRateLimitService = RateLimitService<RedisRateLimiter>;

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
    pub rate_limit: OxidRateLimitService,
    pub rate_limit_quota: Quota,
}

impl OxidService {
    pub fn new(
        auth: OxidAuthService,
        usecase: OxidUseCase,
        rate_limit: OxidRateLimitService,
        rate_limit_quota: Quota,
    ) -> Self {
        Self {
            auth,
            usecase,
            rate_limit,
            rate_limit_quota,
        }
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

    let limiter = RedisRateLimiter::connect(&config.rate_limit.redis_url)
        .await
        .map_err(|e| CoreError::Internal(format!("redis connection failed: {e}")))?;
    let rate_limit = RateLimitService::new(limiter);
    let rate_limit_quota = Quota::per_minute(config.rate_limit.per_minute);

    Ok(OxidService::new(
        auth,
        OxidUseCase::new(pool),
        rate_limit,
        rate_limit_quota,
    ))
}
