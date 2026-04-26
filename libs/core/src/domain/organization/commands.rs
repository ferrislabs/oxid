use crate::{UserId, domain::organization::OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateOrganizationCommand {
    pub name: String,
    pub slug: String,
    pub owner_id: UserId,
}

#[derive(Debug, Clone)]
pub struct UpdateOrganizationCommand {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
}
