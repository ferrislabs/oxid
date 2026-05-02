use common::CoreError;
use oxid_macros::transactional;

use crate::{
    UserId,
    application::OxidUseCase,
    domain::organization::{
        Organization, OrganizationId,
        commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
        service::OrganizationService,
    },
};

impl OxidUseCase {
    #[transactional(organization, role, member, user, authz)]
    pub async fn create_organization(
        &self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.create_organization(command).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn get_organization(&self, id: OrganizationId) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.get_organization(id).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn list_organizations_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Organization>, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.list_organizations_for_user(user_id).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn update_organization(
        &self,
        command: UpdateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.update_organization(command).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn soft_delete_organization(&self, id: OrganizationId) -> Result<(), CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.soft_delete_organization(id).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn leave_organization(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<(), CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.leave_organization(organization_id, user_id).await
    }
}
