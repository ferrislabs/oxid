use common::CoreError;
use oxid_macros::transactional;

use crate::{
    UserId,
    application::OxidUseCase,
    domain::{
        member::{commands::AddMemberCommand, service::MemberService},
        organization::{
            Organization, OrganizationId,
            commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
            service::OrganizationService,
        },
        role::service::RoleService,
    },
    infrastructure::{
        member::postgres::PgMemberRepository,
        organization::postgres::PgOrganizationRepository, role::postgres::PgRoleRepository,
    },
};

impl OxidUseCase {
    #[transactional]
    pub async fn create_organization(
        &self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let owner_id = command.owner_id;

        let organization = {
            let mut service = OrganizationService::new(PgOrganizationRepository::new(tx));
            service.create_organization(command).await?
        };

        let defaults = {
            let mut service = RoleService::new(PgRoleRepository::new(tx));
            service.seed_default_roles(organization.id).await?
        };

        let member = {
            let mut service = MemberService::new(PgMemberRepository::new(tx));
            service
                .add_member(AddMemberCommand {
                    organization_id: organization.id,
                    user_id: owner_id,
                })
                .await?
        };

        {
            let mut service = MemberService::new(PgMemberRepository::new(tx));
            service
                .assign_role(crate::domain::member::commands::AssignRoleCommand {
                    member_id: member.id,
                    role_id: defaults.owner.id,
                })
                .await?;
        }

        Ok(organization)
    }

    #[transactional]
    pub async fn get_organization(
        &self,
        id: OrganizationId,
    ) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(PgOrganizationRepository::new(tx));
        service.get_organization(id).await
    }

    #[transactional]
    pub async fn list_organizations_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Organization>, CoreError> {
        let mut service = OrganizationService::new(PgOrganizationRepository::new(tx));
        service.list_organizations_for_user(user_id).await
    }

    #[transactional]
    pub async fn update_organization(
        &self,
        command: UpdateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(PgOrganizationRepository::new(tx));
        service.update_organization(command).await
    }

    #[transactional]
    pub async fn soft_delete_organization(
        &self,
        id: OrganizationId,
    ) -> Result<(), CoreError> {
        let mut service = OrganizationService::new(PgOrganizationRepository::new(tx));
        service.soft_delete_organization(id).await
    }

    #[transactional]
    pub async fn leave_organization(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<(), CoreError> {
        let organization = {
            let mut service = OrganizationService::new(PgOrganizationRepository::new(tx));
            service.get_organization(organization_id).await?
        };

        if organization.owner_id == user_id {
            let mut service = OrganizationService::new(PgOrganizationRepository::new(tx));
            return service.soft_delete_organization(organization_id).await;
        }

        let member = {
            let mut service = MemberService::new(PgMemberRepository::new(tx));
            service
                .find_membership(organization_id, user_id)
                .await?
                .ok_or(CoreError::NotFound)?
        };

        let mut service = MemberService::new(PgMemberRepository::new(tx));
        service.remove_member(member.id).await
    }
}
