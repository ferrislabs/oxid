//! Fixed-identity adapter, compiled only under the `test-support` feature.
//!
//! Resolves every token to a preset [`Identity`], so tests can drive the HTTP
//! stack without reaching a live identity provider. Gating it behind a feature
//! keeps it out of production builds entirely: an adapter that authenticates
//! unconditionally must not be linkable into the shipped binary.

use std::collections::HashMap;

use crate::{AuthError, Claims, Identity, domain::ports::AuthRepository};

#[derive(Clone, Default)]
pub struct FixedIdentityRepository {
    /// Resolved when the presented token has no entry in `by_token`.
    fallback: Option<Identity>,
    by_token: HashMap<String, Identity>,
}

impl FixedIdentityRepository {
    /// Every presented token resolves to `identity`.
    pub fn authenticating_as(identity: Identity) -> Self {
        Self {
            fallback: Some(identity),
            by_token: HashMap::new(),
        }
    }

    /// Each listed token resolves to its identity; any other token is rejected.
    ///
    /// This is what lets a single API instance — and so a single database —
    /// serve several callers, which is required to exercise cross-tenant access.
    pub fn with_tokens<I, T>(tokens: I) -> Self
    where
        I: IntoIterator<Item = (T, Identity)>,
        T: Into<String>,
    {
        Self {
            fallback: None,
            by_token: tokens
                .into_iter()
                .map(|(token, identity)| (token.into(), identity))
                .collect(),
        }
    }

    /// Every presented token is rejected, as an invalid-token error.
    pub fn rejecting() -> Self {
        Self::default()
    }
}

impl AuthRepository for FixedIdentityRepository {
    async fn validate_token(&self, _token: &str) -> Result<Claims, AuthError> {
        // The adapter carries an already-resolved identity, not a signed token;
        // minting claims would require fabricating a signature it cannot produce.
        Err(AuthError::Internal {
            message: "fixed identity adapter does not produce claims".to_owned(),
        })
    }

    async fn identify(&self, token: &str) -> Result<Identity, AuthError> {
        self.by_token
            .get(token)
            .or(self.fallback.as_ref())
            .cloned()
            .ok_or(AuthError::InvalidToken {
                message: "rejected by the fixed identity adapter".to_owned(),
            })
    }
}
