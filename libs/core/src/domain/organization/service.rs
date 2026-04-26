use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::domain::organization::{
    Organization, OrganizationId, commands::CreateOrganizationCommand, ports::OrganizationRepository,
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
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&organization).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UserId, domain::organization::ports::MockOrganizationRepository};
    use uuid::Uuid;

    fn cmd() -> CreateOrganizationCommand {
        CreateOrganizationCommand {
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: UserId(Uuid::new_v4()),
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
                created_at: o.created_at,
                updated_at: o.updated_at,
            };
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = OrganizationService::new(repo);
        let org = service.create_organization(cmd()).await.unwrap();

        assert_eq!(org.slug, "acme");
        assert_eq!(org.name, "Acme");
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
}
