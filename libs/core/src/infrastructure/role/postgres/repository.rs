use common::CoreError;
use sqlx::{Postgres, Transaction};

use crate::{
    domain::{
        organization::OrganizationId,
        role::{Role, ports::RoleRepository},
    },
    infrastructure::{postgres::error::map_sqlx_error, role::postgres::model::RoleRow},
};

pub struct PgRoleRepository<'tx> {
    tx: &'tx mut Transaction<'static, Postgres>,
}

impl<'tx> PgRoleRepository<'tx> {
    pub fn new(tx: &'tx mut Transaction<'static, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'tx> RoleRepository for PgRoleRepository<'tx> {
    async fn insert(&mut self, role: &Role) -> Result<Role, CoreError> {
        let row = sqlx::query_as!(
            RoleRow,
            r#"
            INSERT INTO roles (id, organization_id, name, permissions, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, name, permissions, created_at, updated_at
            "#,
            role.id.0,
            role.organization_id.0,
            role.name,
            role.permissions,
            role.created_at,
            role.updated_at,
        )
        .fetch_one(&mut **self.tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Role>, CoreError> {
        let rows = sqlx::query_as!(
            RoleRow,
            r#"
            SELECT id, organization_id, name, permissions, created_at, updated_at
            FROM roles
            WHERE organization_id = $1
            ORDER BY created_at ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut **self.tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
