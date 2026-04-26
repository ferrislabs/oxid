use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{UserId, domain::organization::{Organization, OrganizationId}};

#[derive(Debug, Clone)]
pub struct OrganizationRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OrganizationRow> for Organization {
    fn from(row: OrganizationRow) -> Self {
        Self {
            id: OrganizationId(row.id),
            name: row.name,
            slug: row.slug,
            owner_id: UserId(row.owner_id),
            // M2: read deleted_at from DB once the migration + SELECT are added.
            deleted_at: None,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
