use common::CoreError;
use oxid_macros::transactional;

use crate::{
    application::OxidUseCase,
    domain::{
        organization::OrganizationId,
        role::{Role, commands::CreateRoleCommand, service::RoleService},
    },
    infrastructure::role::postgres::PgRoleRepository,
};

impl OxidUseCase {
    #[transactional]
    pub async fn create_role(&self, command: CreateRoleCommand) -> Result<Role, CoreError> {
        let mut service = RoleService::new(PgRoleRepository::new(tx));
        service.create_role(command).await
    }

    #[transactional]
    pub async fn list_roles(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Role>, CoreError> {
        let mut service = RoleService::new(PgRoleRepository::new(tx));
        service.list_roles(organization_id).await
    }
}
