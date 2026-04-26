use crate::domain::organization::OrganizationId;

#[derive(Debug, Clone)]
pub struct CreateRoleCommand {
    pub organization_id: OrganizationId,
    pub name: String,
    pub permissions: i64,
}
