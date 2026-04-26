use common::CoreError;

use crate::domain::organization::Organization;

#[cfg_attr(test, mockall::automock)]
pub trait OrganizationRepository: Send {
    fn insert(
        &mut self,
        organization: &Organization,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;
}
