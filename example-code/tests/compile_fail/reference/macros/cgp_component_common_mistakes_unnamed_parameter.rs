//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: a `_` method parameter fails inside
//! the macro, because the generated impls forward each argument by name.

use cgp::prelude::*;

#[cgp_component(Counter)]
pub trait CanCount {
    fn count(&self, _: u32) -> u32;
}

fn main() {}
