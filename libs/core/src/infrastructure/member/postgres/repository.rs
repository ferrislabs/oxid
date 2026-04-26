use common::CoreError;
use sqlx::{Postgres, Transaction};

use crate::{
    domain::{
        member::{Member, MemberId, ports::MemberRepository},
        organization::OrganizationId,
        role::RoleId,
    },
    infrastructure::{member::postgres::model::MemberRow, postgres::error::map_sqlx_error},
};

pub struct PgMemberRepository<'tx> {
    tx: &'tx mut Transaction<'static, Postgres>,
}

impl<'tx> PgMemberRepository<'tx> {
    pub fn new(tx: &'tx mut Transaction<'static, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'tx> MemberRepository for PgMemberRepository<'tx> {
    async fn insert(&mut self, member: &Member) -> Result<Member, CoreError> {
        let row = sqlx::query_as!(
            MemberRow,
            r#"
            INSERT INTO organization_members (id, organization_id, user_id, joined_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, organization_id, user_id, joined_at
            "#,
            member.id.0,
            member.organization_id.0,
            member.user_id.0,
            member.joined_at,
        )
        .fetch_one(&mut **self.tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Member>, CoreError> {
        let rows = sqlx::query_as!(
            MemberRow,
            r#"
            SELECT id, organization_id, user_id, joined_at
            FROM organization_members
            WHERE organization_id = $1
            ORDER BY joined_at ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut **self.tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn assign_role(
        &mut self,
        member_id: MemberId,
        role_id: RoleId,
    ) -> Result<(), CoreError> {
        sqlx::query!(
            r#"
            INSERT INTO member_roles (id, member_id, role_id)
            VALUES (gen_random_uuid(), $1, $2)
            ON CONFLICT (member_id, role_id) DO NOTHING
            "#,
            member_id.0,
            role_id.0,
        )
        .execute(&mut **self.tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn list_role_ids(&mut self, member_id: MemberId) -> Result<Vec<RoleId>, CoreError> {
        let rows = sqlx::query!(
            r#"
            SELECT role_id
            FROM member_roles
            WHERE member_id = $1
            "#,
            member_id.0,
        )
        .fetch_all(&mut **self.tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| RoleId(r.role_id)).collect())
    }
}
