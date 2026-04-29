use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    UserId,
    domain::{
        member::{Member, MemberId, ports::MemberRepository},
        organization::{
            Organization, OrganizationId,
            commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
            ports::OrganizationRepository,
        },
        role::{
            ADMIN_ROLE_NAME, MEMBER_ROLE_NAME, OWNER_ROLE_NAME, Permissions, Role, RoleId,
            ports::RoleRepository,
        },
        user::ports::UserRepository,
    },
};

pub struct OrganizationService<O, R, M, U>
where
    O: OrganizationRepository,
    R: RoleRepository,
    M: MemberRepository,
    U: UserRepository,
{
    organization_repository: O,
    role_repository: R,
    member_repository: M,
    user_repository: U,
}

impl<O, R, M, U> OrganizationService<O, R, M, U>
where
    O: OrganizationRepository,
    R: RoleRepository,
    M: MemberRepository,
    U: UserRepository,
{
    pub fn new(
        organization_repository: O,
        role_repository: R,
        member_repository: M,
        user_repository: U,
    ) -> Self {
        Self {
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        }
    }

    #[tracing::instrument(skip(self), fields(organization_id = %id.0), err)]
    pub async fn get_organization(
        &mut self,
        id: OrganizationId,
    ) -> Result<Organization, CoreError> {
        self.organization_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    #[tracing::instrument(skip(self), fields(user_id = %user_id.0), err)]
    pub async fn list_organizations_for_user(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<Organization>, CoreError> {
        self.organization_repository.list_for_user(user_id).await
    }

    #[tracing::instrument(skip(self), fields(organization_id = %command.id.0, organization.slug = %command.slug), err)]
    pub async fn update_organization(
        &mut self,
        command: UpdateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let mut organization = self
            .organization_repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        organization.name = command.name;
        organization.slug = command.slug;
        organization.updated_at = Utc::now();

        self.organization_repository.update(&organization).await
    }

    #[tracing::instrument(skip(self), fields(organization_id = %id.0), err)]
    pub async fn soft_delete_organization(&mut self, id: OrganizationId) -> Result<(), CoreError> {
        self.organization_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        self.organization_repository
            .soft_delete(id, Utc::now())
            .await
    }

    #[tracing::instrument(skip(self), fields(organization.slug = %command.slug, owner_id = %command.owner_id.0), err)]
    pub async fn create_organization(
        &mut self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let now = Utc::now();
        let owner_id = command.owner_id;

        let user = self
            .user_repository
            .find_by_sub(owner_id.to_string().as_str())
            .await?
            .ok_or(CoreError::NotFound)?;

        let organization = self
            .organization_repository
            .insert(&Organization {
                id: OrganizationId(generate_uuid_v7()),
                name: command.name,
                slug: command.slug,
                owner_id: user.id,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        let owner_role = self
            .role_repository
            .insert(&Role {
                id: RoleId(generate_uuid_v7()),
                organization_id: organization.id,
                name: OWNER_ROLE_NAME.into(),
                permissions: Permissions::ALL,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.role_repository
            .insert(&Role {
                id: RoleId(generate_uuid_v7()),
                organization_id: organization.id,
                name: ADMIN_ROLE_NAME.into(),
                permissions: Permissions::MANAGE_MEMBERS,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.role_repository
            .insert(&Role {
                id: RoleId(generate_uuid_v7()),
                organization_id: organization.id,
                name: MEMBER_ROLE_NAME.into(),
                permissions: Permissions::NONE,
                created_at: now,
                updated_at: now,
            })
            .await?;

        let member = self
            .member_repository
            .insert(&Member {
                id: MemberId(generate_uuid_v7()),
                organization_id: organization.id,
                user_id: user.id,
                joined_at: now,
            })
            .await?;

        self.member_repository
            .assign_role(member.id, owner_role.id)
            .await?;

        Ok(organization)
    }

    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0, user_id = %user_id.0), err)]
    pub async fn leave_organization(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<(), CoreError> {
        let organization = self.get_organization(organization_id).await?;

        if organization.owner_id == user_id {
            return self.soft_delete_organization(organization_id).await;
        }

        let member = self
            .member_repository
            .find_by_org_and_user(organization_id, user_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        self.member_repository.remove(member.id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UserId, domain::organization::ports::MockOrganizationRepository};
    use chrono::DateTime;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn fixture(id: OrganizationId) -> Organization {
        let now = Utc::now();
        Organization {
            id,
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: UserId(Uuid::new_v4()),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn get_organization_returns_not_found_when_missing() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );
        let err = service.get_organization(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn get_organization_returns_entity_when_found() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        let org = service.get_organization(id).await.unwrap();

        assert_eq!(org.id, id);
    }

    #[tokio::test]
    async fn update_organization_mutates_and_saves() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });
        organization_repository
            .expect_update()
            .times(1)
            .returning(|o| {
                let cloned = Organization {
                    id: o.id,
                    name: o.name.clone(),
                    slug: o.slug.clone(),
                    owner_id: o.owner_id,
                    deleted_at: o.deleted_at,
                    created_at: o.created_at,
                    updated_at: o.updated_at,
                };
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        let updated = service
            .update_organization(UpdateOrganizationCommand {
                id,
                name: "Acme Inc.".into(),
                slug: "acme-inc".into(),
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Acme Inc.");
        assert_eq!(updated.slug, "acme-inc");
    }

    #[tokio::test]
    async fn update_organization_returns_not_found_when_missing() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        let err = service
            .update_organization(UpdateOrganizationCommand {
                id,
                name: "Whatever".into(),
                slug: "whatever".into(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn soft_delete_organization_calls_repo() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });

        organization_repository
            .expect_soft_delete()
            .withf(move |i, _| *i == id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        service.soft_delete_organization(id).await.unwrap();
    }

    #[tokio::test]
    async fn soft_delete_organization_returns_not_found_when_missing() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        let err = service.soft_delete_organization(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_organizations_for_user_delegates_to_repo() {
        let user_id = UserId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_list_for_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        let orgs = service.list_organizations_for_user(user_id).await.unwrap();

        assert!(orgs.is_empty());
    }

    use crate::{
        User,
        domain::{
            member::ports::MockMemberRepository, role::ports::MockRoleRepository,
            user::ports::MockUserRepository,
        },
    };

    struct MockRepos {
        org: MockOrganizationRepository,
        role: MockRoleRepository,
        member: MockMemberRepository,
        user: MockUserRepository,
    }

    impl MockRepos {
        fn new() -> Self {
            Self {
                org: MockOrganizationRepository::new(),
                role: MockRoleRepository::new(),
                member: MockMemberRepository::new(),
                user: MockUserRepository::new(),
            }
        }
    }

    impl OrganizationRepository for MockRepos {
        async fn insert(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
            self.org.insert(organization).await
        }

        async fn find_by_id(
            &mut self,
            id: OrganizationId,
        ) -> Result<Option<Organization>, CoreError> {
            self.org.find_by_id(id).await
        }

        async fn list_for_user(&mut self, user_id: UserId) -> Result<Vec<Organization>, CoreError> {
            self.org.list_for_user(user_id).await
        }

        async fn update(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
            self.org.update(organization).await
        }

        async fn soft_delete(
            &mut self,
            id: OrganizationId,
            deleted_at: DateTime<Utc>,
        ) -> Result<(), CoreError> {
            self.org.soft_delete(id, deleted_at).await
        }
    }

    impl RoleRepository for MockRepos {
        async fn insert(&mut self, role: &Role) -> Result<Role, CoreError> {
            self.role.insert(role).await
        }
        async fn list_by_organization(
            &mut self,
            organization_id: OrganizationId,
        ) -> Result<Vec<Role>, CoreError> {
            self.role.list_by_organization(organization_id).await
        }
    }

    impl MemberRepository for MockRepos {
        async fn insert(&mut self, member: &Member) -> Result<Member, CoreError> {
            self.member.insert(member).await
        }

        async fn list_by_organization(
            &mut self,
            organization_id: OrganizationId,
        ) -> Result<Vec<Member>, CoreError> {
            self.member.list_by_organization(organization_id).await
        }

        async fn assign_role(
            &mut self,
            member_id: MemberId,
            role_id: RoleId,
        ) -> Result<(), CoreError> {
            self.member.assign_role(member_id, role_id).await
        }

        async fn find_by_org_and_user(
            &mut self,
            organization_id: OrganizationId,
            user_id: UserId,
        ) -> Result<Option<Member>, CoreError> {
            self.member
                .find_by_org_and_user(organization_id, user_id)
                .await
        }

        async fn remove(&mut self, member_id: MemberId) -> Result<(), CoreError> {
            self.member.remove(member_id).await
        }

        async fn list_role_ids(&mut self, member_id: MemberId) -> Result<Vec<RoleId>, CoreError> {
            self.member.list_role_ids(member_id).await
        }
    }

    impl UserRepository for MockRepos {
        async fn upsert_by_email(&mut self, user: &User) -> Result<User, CoreError> {
            self.user.upsert_by_email(user).await
        }

        async fn find_by_email(&mut self, email: &str) -> Result<Option<User>, CoreError> {
            self.user.find_by_email(email).await
        }

        async fn find_by_sub(&mut self, sub: &str) -> Result<Option<User>, CoreError> {
            self.user.find_by_sub(sub).await
        }
    }

    fn create_cmd() -> CreateOrganizationCommand {
        CreateOrganizationCommand {
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: UserId(Uuid::new_v4()),
        }
    }

    #[tokio::test]
    async fn create_organization_seeds_roles_and_owner_membership() {
        let mut organization_repository = MockOrganizationRepository::new();
        let mut role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_insert()
            .times(1)
            .returning(|o| {
                let cloned = Organization {
                    id: o.id,
                    name: o.name.clone(),
                    slug: o.slug.clone(),
                    owner_id: o.owner_id,
                    deleted_at: o.deleted_at,
                    created_at: o.created_at,
                    updated_at: o.updated_at,
                };
                Box::pin(async move { Ok(cloned) })
            });

        role_repository.expect_insert().times(3).returning(|r| {
            let cloned = Role {
                id: r.id,
                organization_id: r.organization_id,
                name: r.name.clone(),
                permissions: r.permissions,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            Box::pin(async move { Ok(cloned) })
        });

        member_repository.expect_insert().times(1).returning(|m| {
            let cloned = Member {
                id: m.id,
                organization_id: m.organization_id,
                user_id: m.user_id,
                joined_at: m.joined_at,
            };
            Box::pin(async move { Ok(cloned) })
        });

        member_repository
            .expect_assign_role()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        let org = service.create_organization(create_cmd()).await.unwrap();

        assert_eq!(org.name, "Acme");
        assert_eq!(org.slug, "acme");
        assert!(org.deleted_at.is_none());
    }

    #[tokio::test]
    async fn leave_organization_soft_deletes_when_owner_leaves() {
        let owner_id = UserId(Uuid::new_v4());
        let org_id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(org_id))
            .times(2)
            .returning(move |id| {
                let now = Utc::now();
                let org = Organization {
                    id,
                    name: "Acme".into(),
                    slug: "acme".into(),
                    owner_id,
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(org)) })
            });

        organization_repository
            .expect_soft_delete()
            .withf(move |i, _| *i == org_id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        service.leave_organization(org_id, owner_id).await.unwrap();
    }

    #[tokio::test]
    async fn leave_organization_removes_membership_when_non_owner_leaves() {
        let owner_id = UserId(Uuid::new_v4());
        let leaver_id = UserId(Uuid::new_v4());
        let org_id = OrganizationId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(org_id))
            .times(1)
            .returning(move |id| {
                let now = Utc::now();
                let org = Organization {
                    id,
                    name: "Acme".into(),
                    slug: "acme".into(),
                    owner_id,
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(org)) })
            });

        member_repository
            .expect_find_by_org_and_user()
            .with(eq(org_id), eq(leaver_id))
            .times(1)
            .returning(move |organization_id, user_id| {
                let m = Member {
                    id: member_id,
                    organization_id,
                    user_id,
                    joined_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(m)) })
            });

        member_repository
            .expect_remove()
            .with(eq(member_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
        );

        service.leave_organization(org_id, leaver_id).await.unwrap();
    }
}
