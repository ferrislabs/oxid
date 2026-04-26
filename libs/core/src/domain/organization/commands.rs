use crate::UserId;

#[derive(Debug, Clone)]
pub struct CreateOrganizationCommand {
    pub name: String,
    pub slug: String,
    pub owner_id: UserId,
}
