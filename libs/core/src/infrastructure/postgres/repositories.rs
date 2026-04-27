use chrono::{DateTime, Utc};
use common::CoreError;
use sqlx::{Postgres, Transaction};

use crate::{
    User, UserId,
    domain::{
        member::{Member, MemberId, ports::MemberRepository},
        organization::{Organization, OrganizationId, ports::OrganizationRepository},
        role::{Role, RoleId, ports::RoleRepository},
        user::ports::UserRepository,
    },
    infrastructure::{
        member::postgres::PgMemberRepository, organization::postgres::PgOrganizationRepository,
        role::postgres::PgRoleRepository, user::postgres::PgUserRepository,
    },
};

pub struct PgRepositories<'tx> {
    tx: &'tx mut Transaction<'static, Postgres>,
}

impl<'tx> PgRepositories<'tx> {
    pub fn new(tx: &'tx mut Transaction<'static, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'tx> OrganizationRepository for PgRepositories<'tx> {
    async fn insert(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
        PgOrganizationRepository::new(&mut *self.tx)
            .insert(organization)
            .await
    }

    async fn find_by_id(&mut self, id: OrganizationId) -> Result<Option<Organization>, CoreError> {
        PgOrganizationRepository::new(&mut *self.tx)
            .find_by_id(id)
            .await
    }

    async fn list_for_user(&mut self, user_id: UserId) -> Result<Vec<Organization>, CoreError> {
        PgOrganizationRepository::new(&mut *self.tx)
            .list_for_user(user_id)
            .await
    }

    async fn update(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
        PgOrganizationRepository::new(&mut *self.tx)
            .update(organization)
            .await
    }

    async fn soft_delete(
        &mut self,
        id: OrganizationId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        PgOrganizationRepository::new(&mut *self.tx)
            .soft_delete(id, deleted_at)
            .await
    }
}

impl<'tx> RoleRepository for PgRepositories<'tx> {
    async fn insert(&mut self, role: &Role) -> Result<Role, CoreError> {
        PgRoleRepository::new(&mut *self.tx).insert(role).await
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Role>, CoreError> {
        PgRoleRepository::new(&mut *self.tx)
            .list_by_organization(organization_id)
            .await
    }
}

impl<'tx> MemberRepository for PgRepositories<'tx> {
    async fn insert(&mut self, member: &Member) -> Result<Member, CoreError> {
        PgMemberRepository::new(&mut *self.tx).insert(member).await
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Member>, CoreError> {
        PgMemberRepository::new(&mut *self.tx)
            .list_by_organization(organization_id)
            .await
    }

    async fn find_by_org_and_user(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Member>, CoreError> {
        PgMemberRepository::new(&mut *self.tx)
            .find_by_org_and_user(organization_id, user_id)
            .await
    }

    async fn remove(&mut self, member_id: MemberId) -> Result<(), CoreError> {
        PgMemberRepository::new(&mut *self.tx)
            .remove(member_id)
            .await
    }

    async fn assign_role(&mut self, member_id: MemberId, role_id: RoleId) -> Result<(), CoreError> {
        PgMemberRepository::new(&mut *self.tx)
            .assign_role(member_id, role_id)
            .await
    }

    async fn list_role_ids(&mut self, member_id: MemberId) -> Result<Vec<RoleId>, CoreError> {
        PgMemberRepository::new(&mut *self.tx)
            .list_role_ids(member_id)
            .await
    }
}

impl<'tx> UserRepository for PgRepositories<'tx> {
    async fn upsert_by_email(&mut self, user: &User) -> Result<User, CoreError> {
        PgUserRepository::new(&mut *self.tx)
            .upsert_by_email(user)
            .await
    }

    async fn find_by_email(&mut self, email: &str) -> Result<Option<User>, CoreError> {
        PgUserRepository::new(&mut *self.tx)
            .find_by_email(email)
            .await
    }
}
