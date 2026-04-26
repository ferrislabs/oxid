use common::CoreError;
use oxid_macros::transactional;

use crate::{
    application::OxidUseCase,
    domain::organization::{
        Organization, commands::CreateOrganizationCommand, service::OrganizationService,
    },
    infrastructure::organization::postgres::PgOrganizationRepository,
};

impl OxidUseCase {
    #[transactional]
    pub async fn create_organization(
        &self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(PgOrganizationRepository::new(tx));
        service.create_organization(command).await
    }
}
