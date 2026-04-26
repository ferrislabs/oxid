use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    UserId,
    domain::organization::{
        Organization, OrganizationId,
        commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
        ports::OrganizationRepository,
    },
};

pub struct OrganizationService<R>
where
    R: OrganizationRepository,
{
    repo: R,
}

impl<R> OrganizationService<R>
where
    R: OrganizationRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_organization(
        &mut self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let now = Utc::now();
        let organization = Organization {
            id: OrganizationId(generate_uuid_v7()),
            name: command.name,
            slug: command.slug,
            owner_id: command.owner_id,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&organization).await
    }

    pub async fn get_organization(
        &mut self,
        id: OrganizationId,
    ) -> Result<Organization, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_organizations_for_user(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<Organization>, CoreError> {
        self.repo.list_for_user(user_id).await
    }

    pub async fn update_organization(
        &mut self,
        command: UpdateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let mut organization = self
            .repo
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        organization.name = command.name;
        organization.slug = command.slug;
        organization.updated_at = Utc::now();

        self.repo.update(&organization).await
    }

    pub async fn soft_delete_organization(
        &mut self,
        id: OrganizationId,
    ) -> Result<(), CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        self.repo.soft_delete(id, Utc::now()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UserId, domain::organization::ports::MockOrganizationRepository};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn cmd() -> CreateOrganizationCommand {
        CreateOrganizationCommand {
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: UserId(Uuid::new_v4()),
        }
    }

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
    async fn create_organization_persists_via_repo() {
        let mut repo = MockOrganizationRepository::new();
        repo.expect_insert().times(1).returning(|o| {
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

        let mut service = OrganizationService::new(repo);
        let org = service.create_organization(cmd()).await.unwrap();

        assert_eq!(org.slug, "acme");
        assert_eq!(org.name, "Acme");
        assert!(org.deleted_at.is_none());
    }

    #[tokio::test]
    async fn create_organization_propagates_repo_error() {
        let mut repo = MockOrganizationRepository::new();
        repo.expect_insert()
            .returning(|_| Box::pin(async { Err(CoreError::Conflict("slug taken".into())) }));

        let mut service = OrganizationService::new(repo);

        let err = service.create_organization(cmd()).await.unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn get_organization_returns_not_found_when_missing() {
        let id = OrganizationId(Uuid::new_v4());
        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(repo);
        let err = service.get_organization(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn get_organization_returns_entity_when_found() {
        let id = OrganizationId(Uuid::new_v4());
        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });

        let mut service = OrganizationService::new(repo);
        let org = service.get_organization(id).await.unwrap();

        assert_eq!(org.id, id);
    }

    #[tokio::test]
    async fn update_organization_mutates_and_saves() {
        let id = OrganizationId(Uuid::new_v4());
        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });
        repo.expect_update().times(1).returning(|o| {
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

        let mut service = OrganizationService::new(repo);
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
        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(repo);
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
        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });
        repo.expect_soft_delete()
            .withf(move |i, _| *i == id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = OrganizationService::new(repo);
        service.soft_delete_organization(id).await.unwrap();
    }

    #[tokio::test]
    async fn soft_delete_organization_returns_not_found_when_missing() {
        let id = OrganizationId(Uuid::new_v4());
        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(repo);
        let err = service.soft_delete_organization(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_organizations_for_user_delegates_to_repo() {
        let user_id = UserId(Uuid::new_v4());
        let mut repo = MockOrganizationRepository::new();
        repo.expect_list_for_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let mut service = OrganizationService::new(repo);
        let orgs = service.list_organizations_for_user(user_id).await.unwrap();

        assert!(orgs.is_empty());
    }
}
