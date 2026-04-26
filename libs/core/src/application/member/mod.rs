use common::CoreError;
use oxid_macros::transactional;

use crate::{
    UserId,
    application::OxidUseCase,
    domain::{
        member::{
            Member, MemberId,
            commands::{AddMemberCommand, AssignRoleCommand},
            service::MemberService,
        },
        organization::OrganizationId,
        role::RoleId,
    },
    infrastructure::member::postgres::PgMemberRepository,
};

impl OxidUseCase {
    #[transactional]
    pub async fn add_member(&self, command: AddMemberCommand) -> Result<Member, CoreError> {
        let mut service = MemberService::new(PgMemberRepository::new(tx));
        service.add_member(command).await
    }

    #[transactional]
    pub async fn list_members(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Member>, CoreError> {
        let mut service = MemberService::new(PgMemberRepository::new(tx));
        service.list_members(organization_id).await
    }

    #[transactional]
    pub async fn find_membership(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Member>, CoreError> {
        let mut service = MemberService::new(PgMemberRepository::new(tx));
        service.find_membership(organization_id, user_id).await
    }

    #[transactional]
    pub async fn remove_member(&self, member_id: MemberId) -> Result<(), CoreError> {
        let mut service = MemberService::new(PgMemberRepository::new(tx));
        service.remove_member(member_id).await
    }

    #[transactional]
    pub async fn assign_role(&self, command: AssignRoleCommand) -> Result<(), CoreError> {
        let mut service = MemberService::new(PgMemberRepository::new(tx));
        service.assign_role(command).await
    }

    #[transactional]
    pub async fn list_role_ids(&self, member_id: MemberId) -> Result<Vec<RoleId>, CoreError> {
        let mut service = MemberService::new(PgMemberRepository::new(tx));
        service.list_role_ids(member_id).await
    }
}
