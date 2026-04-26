use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::domain::{
    organization::OrganizationId,
    role::{
        ADMIN_ROLE_NAME, MEMBER_ROLE_NAME, OWNER_ROLE_NAME, Permissions, Role, RoleId,
        commands::CreateRoleCommand, ports::RoleRepository,
    },
};

#[derive(Debug, Clone)]
pub struct DefaultRoles {
    pub owner: Role,
    pub admin: Role,
    pub member: Role,
}

pub struct RoleService<R>
where
    R: RoleRepository,
{
    repo: R,
}

impl<R> RoleService<R>
where
    R: RoleRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_role(&mut self, command: CreateRoleCommand) -> Result<Role, CoreError> {
        let now = Utc::now();
        let role = Role {
            id: RoleId(generate_uuid_v7()),
            organization_id: command.organization_id,
            name: command.name,
            permissions: command.permissions,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&role).await
    }

    pub async fn list_roles(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Role>, CoreError> {
        self.repo.list_by_organization(organization_id).await
    }

    pub async fn seed_default_roles(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<DefaultRoles, CoreError> {
        let owner = self
            .create_role(CreateRoleCommand {
                organization_id,
                name: OWNER_ROLE_NAME.into(),
                permissions: Permissions::ALL,
            })
            .await?;

        let admin = self
            .create_role(CreateRoleCommand {
                organization_id,
                name: ADMIN_ROLE_NAME.into(),
                permissions: Permissions::MANAGE_MEMBERS,
            })
            .await?;

        let member = self
            .create_role(CreateRoleCommand {
                organization_id,
                name: MEMBER_ROLE_NAME.into(),
                permissions: Permissions::NONE,
            })
            .await?;

        Ok(DefaultRoles {
            owner,
            admin,
            member,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::role::ports::MockRoleRepository;
    use chrono::Utc;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn org_id() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    #[tokio::test]
    async fn create_role_persists_via_repo() {
        let mut repo = MockRoleRepository::new();
        repo.expect_insert().times(1).returning(|r| {
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

        let mut service = RoleService::new(repo);
        let role = service
            .create_role(CreateRoleCommand {
                organization_id: org_id(),
                name: "admin".into(),
                permissions: Permissions::MANAGE_MEMBERS,
            })
            .await
            .unwrap();

        assert_eq!(role.name, "admin");
        assert_eq!(role.permissions, Permissions::MANAGE_MEMBERS);
    }

    #[tokio::test]
    async fn list_roles_delegates_to_repo() {
        let id = org_id();
        let mut repo = MockRoleRepository::new();
        repo.expect_list_by_organization()
            .with(eq(id))
            .times(1)
            .returning(move |oid| {
                let now = Utc::now();
                let roles = vec![Role {
                    id: RoleId(Uuid::new_v4()),
                    organization_id: oid,
                    name: "admin".into(),
                    permissions: Permissions::MANAGE_MEMBERS,
                    created_at: now,
                    updated_at: now,
                }];
                Box::pin(async move { Ok(roles) })
            });

        let mut service = RoleService::new(repo);
        let roles = service.list_roles(id).await.unwrap();

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "admin");
    }

    #[tokio::test]
    async fn seed_default_roles_creates_owner_admin_member() {
        let oid = org_id();
        let mut repo = MockRoleRepository::new();

        // Three sequential inserts: owner, admin, member.
        repo.expect_insert()
            .times(3)
            .returning(|r| {
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

        let mut service = RoleService::new(repo);
        let defaults = service.seed_default_roles(oid).await.unwrap();

        assert_eq!(defaults.owner.name, OWNER_ROLE_NAME);
        assert_eq!(defaults.owner.permissions, Permissions::ALL);
        assert_eq!(defaults.admin.name, ADMIN_ROLE_NAME);
        assert_eq!(defaults.admin.permissions, Permissions::MANAGE_MEMBERS);
        assert_eq!(defaults.member.name, MEMBER_ROLE_NAME);
        assert!(defaults.member.permissions.is_empty());

        assert_eq!(defaults.owner.organization_id, oid);
        assert_eq!(defaults.admin.organization_id, oid);
        assert_eq!(defaults.member.organization_id, oid);
    }
}
