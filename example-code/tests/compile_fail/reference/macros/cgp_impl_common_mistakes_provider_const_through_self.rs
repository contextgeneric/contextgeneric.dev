//! `docs/reference/macros/cgp_impl.md`, *Common Mistakes*: `Self::LIMIT` naming the provider's own
//! associated const is rewritten to the context, which lacks it.

use cgp::prelude::*;

#[cgp_component(RateLimiter)]
pub trait CanRateLimit {
    const LIMIT: u64;

    fn allows(&self, count: u64) -> bool;
}

#[cgp_impl(new AllowUnderLimit)]
impl RateLimiter {
    const LIMIT: u64 = 100;

    fn allows(&self, count: u64) -> bool {
        count < Self::LIMIT
    }
}

fn main() {}
