//! `docs/reference/macros/cgp_provider.md`, *Input the macros refuse*: a provider trait with no
//! type argument leaves nothing to be the context.

use cgp::prelude::*;

pub trait PlainTrait {
    fn value() -> u32;
}

pub struct Provider;

#[cgp_provider(SomeComponent)]
impl PlainTrait for Provider {
    fn value() -> u32 {
        1
    }
}

fn main() {}
