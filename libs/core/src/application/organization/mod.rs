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
    infrastructure::{
        member::postgres::PgMemberRepository, organization::postgres::PgOrganizationRepository,
        role::postgres::PgRoleRepository,
    },
};

impl OxidUseCase {
    #[transactional]
    pub async fn create_organization(
        &self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let organization_repository = PgOrganizationRepository::new(&tx);
        let role_repository = PgRoleRepository::new(&tx);
        let member_repository = PgMemberRepository::new(&tx);

        let mut service =
            OrganizationService::new(organization_repository, role_repository, member_repository);
        service.create_organization(command).await
    }

    #[transactional]
    pub async fn get_organization(&self, id: OrganizationId) -> Result<Organization, CoreError> {
        let organization_repository = PgOrganizationRepository::new(&tx);
        let role_repository = PgRoleRepository::new(&tx);
        let member_repository = PgMemberRepository::new(&tx);

        let mut service =
            OrganizationService::new(organization_repository, role_repository, member_repository);
        service.get_organization(id).await
    }

    #[transactional]
    pub async fn list_organizations_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Organization>, CoreError> {
        let organization_repository = PgOrganizationRepository::new(&tx);
        let role_repository = PgRoleRepository::new(&tx);
        let member_repository = PgMemberRepository::new(&tx);

        let mut service =
            OrganizationService::new(organization_repository, role_repository, member_repository);
        service.list_organizations_for_user(user_id).await
    }

    #[transactional]
    pub async fn update_organization(
        &self,
        command: UpdateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let organization_repository = PgOrganizationRepository::new(&tx);
        let role_repository = PgRoleRepository::new(&tx);
        let member_repository = PgMemberRepository::new(&tx);

        let mut service =
            OrganizationService::new(organization_repository, role_repository, member_repository);
        service.update_organization(command).await
    }

    #[transactional]
    pub async fn soft_delete_organization(&self, id: OrganizationId) -> Result<(), CoreError> {
        let organization_repository = PgOrganizationRepository::new(&tx);
        let role_repository = PgRoleRepository::new(&tx);
        let member_repository = PgMemberRepository::new(&tx);

        let mut service =
            OrganizationService::new(organization_repository, role_repository, member_repository);
        service.soft_delete_organization(id).await
    }

    #[transactional]
    pub async fn leave_organization(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<(), CoreError> {
        let organization_repository = PgOrganizationRepository::new(&tx);
        let role_repository = PgRoleRepository::new(&tx);
        let member_repository = PgMemberRepository::new(&tx);

        let mut service =
            OrganizationService::new(organization_repository, role_repository, member_repository);
        service.leave_organization(organization_id, user_id).await
    }
}
