//! Domain abstraction over an external Identity & Access Management (IAM)
//! provider (e.g. Ferriskey, Keycloak).
//!
//! This crate currently exposes the **domain layer only**: a port trait
//! ([`IamProvider`]) expressed in terms of domain DTOs ([`IamUser`],
//! [`IamOrganization`], [`IamRole`]) and a typed error ([`IamError`]).
//! Concrete adapters live in separate crates so that use-cases stay
//! decoupled from the wire protocol of any specific IAM.
//!
//! See issue #19 for the broader design context.

pub mod domain;

pub use domain::{
    errors::IamError,
    models::{
        organization::{IamCreateOrganization, IamOrganization, IamOrgId, IamUpdateOrganization},
        role::{IamCreateRole, IamRole, IamRoleId, IamUpdateRole},
        user::{IamCreateUser, IamUpdateUser, IamUser, IamUserId},
    },
    ports::IamProvider,
};
