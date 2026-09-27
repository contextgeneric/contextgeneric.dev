//! `docs/reference/macros/cgp_provider.md`, *Common Mistakes*: `Self::LIMIT` in a raw provider
//! impl is ambiguous, because the provider struct also gets the consumer trait.

use cgp::prelude::*;

#[cgp_component(RateLimiter)]
pub trait CanRateLimit {
    const LIMIT: u64;

    fn allowed(&self) -> bool;
}

pub struct AllowUnderLimit;

#[cgp_provider]
impl<Context> RateLimiter<Context> for AllowUnderLimit {
    const LIMIT: u64 = 100;

    fn allowed(_context: &Context) -> bool {
        Self::LIMIT > 0
    }
}

fn main() {}
