use crate::domain::{
    errors::IamError,
    models::{
        organization::{IamCreateOrganization, IamOrgId, IamOrganization, IamUpdateOrganization},
        role::{IamCreateRole, IamRole, IamRoleId, IamUpdateRole},
        user::{IamCreateUser, IamUpdateUser, IamUser, IamUserId},
    },
};

/// Domain port for an external Identity & Access Management provider.
///
/// All methods are expressed in terms of the IAM's own identifiers
/// ([`IamUserId`], [`IamOrgId`], [`IamRoleId`]) — Oxid's database ids are
/// never passed through this trait. Mapping `(IamUserId ↔ UserId)` lives in
/// the application layer.
///
/// Implementations must be safe to share across tasks (`Send + Sync`).
/// Calls are expected to be **idempotent** when the inputs allow it
/// (creates with a stable external id, deletes of an unknown id, ...).
pub trait Iam: Send + Sync {
    // --- Users -----------------------------------------------------------

    fn create_user(
        &self,
        command: IamCreateUser,
    ) -> impl Future<Output = Result<IamUser, IamError>> + Send;

    fn update_user(
        &self,
        id: &IamUserId,
        command: IamUpdateUser,
    ) -> impl Future<Output = Result<IamUser, IamError>> + Send;

    fn delete_user(
        &self,
        id: &IamUserId,
    ) -> impl Future<Output = Result<(), IamError>> + Send;

    fn find_user(
        &self,
        id: &IamUserId,
    ) -> impl Future<Output = Result<Option<IamUser>, IamError>> + Send;

    fn find_user_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<IamUser>, IamError>> + Send;

    // --- Organizations ---------------------------------------------------

    fn create_organization(
        &self,
        command: IamCreateOrganization,
    ) -> impl Future<Output = Result<IamOrganization, IamError>> + Send;

    fn update_organization(
        &self,
        id: &IamOrgId,
        command: IamUpdateOrganization,
    ) -> impl Future<Output = Result<IamOrganization, IamError>> + Send;

    fn delete_organization(
        &self,
        id: &IamOrgId,
    ) -> impl Future<Output = Result<(), IamError>> + Send;

    fn find_organization(
        &self,
        id: &IamOrgId,
    ) -> impl Future<Output = Result<Option<IamOrganization>, IamError>> + Send;

    // --- Membership ------------------------------------------------------

    fn add_user_to_organization(
        &self,
        user: &IamUserId,
        organization: &IamOrgId,
    ) -> impl Future<Output = Result<(), IamError>> + Send;

    fn remove_user_from_organization(
        &self,
        user: &IamUserId,
        organization: &IamOrgId,
    ) -> impl Future<Output = Result<(), IamError>> + Send;

    // --- Roles -----------------------------------------------------------

    fn create_role(
        &self,
        organization: &IamOrgId,
        command: IamCreateRole,
    ) -> impl Future<Output = Result<IamRole, IamError>> + Send;

    fn update_role(
        &self,
        id: &IamRoleId,
        command: IamUpdateRole,
    ) -> impl Future<Output = Result<IamRole, IamError>> + Send;

    fn delete_role(
        &self,
        id: &IamRoleId,
    ) -> impl Future<Output = Result<(), IamError>> + Send;

    fn list_roles(
        &self,
        organization: &IamOrgId,
    ) -> impl Future<Output = Result<Vec<IamRole>, IamError>> + Send;

    fn assign_role(
        &self,
        user: &IamUserId,
        role: &IamRoleId,
    ) -> impl Future<Output = Result<(), IamError>> + Send;

    fn unassign_role(
        &self,
        user: &IamUserId,
        role: &IamRoleId,
    ) -> impl Future<Output = Result<(), IamError>> + Send;
}
