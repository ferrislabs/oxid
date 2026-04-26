use common::CoreError;

use crate::domain::{
    member::{Member, MemberId},
    organization::OrganizationId,
    role::RoleId,
};

#[cfg_attr(test, mockall::automock)]
pub trait MemberRepository: Send {
    fn insert(
        &mut self,
        member: &Member,
    ) -> impl Future<Output = Result<Member, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Member>, CoreError>> + Send;

    fn assign_role(
        &mut self,
        member_id: MemberId,
        role_id: RoleId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_role_ids(
        &mut self,
        member_id: MemberId,
    ) -> impl Future<Output = Result<Vec<RoleId>, CoreError>> + Send;
}
