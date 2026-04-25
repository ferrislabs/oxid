use auth::{AuthService, FerrisKeyRepository};
use common::{Config, CoreError};

pub type OxidAuthService = AuthService<FerrisKeyRepository>;

#[derive(Clone)]
pub struct OxidService {
    pub auth: OxidAuthService,
}

impl OxidService {
    pub fn new(auth: OxidAuthService) -> Self {
        Self { auth }
    }
}

pub async fn create_service(config: Config) -> Result<OxidService, CoreError> {
    let auth_repo = FerrisKeyRepository::new(config.auth.issuer, None);
    let auth = AuthService::new(auth_repo);

    let service = OxidService::new(auth);

    Ok(service)
}
