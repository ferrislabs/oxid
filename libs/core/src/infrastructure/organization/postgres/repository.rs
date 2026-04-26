use common::CoreError;
use sqlx::{Postgres, Transaction};

use crate::{
    domain::organization::{Organization, ports::OrganizationRepository},
    infrastructure::{
        organization::postgres::model::OrganizationRow, postgres::error::map_sqlx_error,
    },
};

pub struct PgOrganizationRepository<'tx> {
    tx: &'tx mut Transaction<'static, Postgres>,
}

impl<'tx> PgOrganizationRepository<'tx> {
    pub fn new(tx: &'tx mut Transaction<'static, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'tx> OrganizationRepository for PgOrganizationRepository<'tx> {
    async fn insert(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
        let row = sqlx::query_as!(
            OrganizationRow,
            r#"
            INSERT INTO organizations (id, name, slug, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, slug, owner_id, created_at, updated_at
            "#,
            organization.id.0,
            organization.name,
            organization.slug,
            organization.owner_id.0,
            organization.created_at,
            organization.updated_at,
        )
        .fetch_one(&mut **self.tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

}
