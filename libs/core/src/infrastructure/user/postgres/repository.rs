use common::CoreError;
use sqlx::{Postgres, Transaction};

use crate::{
    User,
    domain::user::ports::UserRepository,
    infrastructure::{postgres::error::map_sqlx_error, user::postgres::model::UserRow},
};

pub struct PgUserRepository<'tx> {
    tx: &'tx mut Transaction<'static, Postgres>,
}

impl<'tx> PgUserRepository<'tx> {
    pub fn new(tx: &'tx mut Transaction<'static, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'tx> UserRepository for PgUserRepository<'tx> {
    async fn upsert_by_email(&mut self, user: &User) -> Result<User, CoreError> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (id, email, username, display_name, sub, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (email) DO UPDATE SET
                username     = EXCLUDED.username,
                display_name = EXCLUDED.display_name,
                sub          = EXCLUDED.sub,
                updated_at   = EXCLUDED.updated_at
            RETURNING id, email, username, display_name, sub, created_at, updated_at
            "#,
            user.id.0,
            user.email,
            user.username,
            user.name,
            user.sub,
            user.created_at,
            user.updated_at,
        )
        .fetch_one(&mut **self.tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }
}
