//! Always-allow limiter, compiled only under the `test-support` feature.
//!
//! Lets HTTP-level tests exercise routing and authorization without standing up
//! Redis. The limiter's own behaviour is covered against a real backend in
//! `tests/redis_integration.rs`; duplicating it here would only slow the suite.

use crate::{
    Quota, RateLimitDecision, RateLimitError, RateLimitKey, domain::ports::RateLimiter,
};

#[derive(Clone, Copy, Default)]
pub struct AlwaysAllowLimiter;

impl RateLimiter for AlwaysAllowLimiter {
    async fn check(
        &self,
        _key: &RateLimitKey,
        quota: Quota,
    ) -> Result<RateLimitDecision, RateLimitError> {
        Ok(RateLimitDecision {
            allowed: true,
            limit: quota.limit,
            remaining: quota.limit,
            retry_after: None,
        })
    }
}
