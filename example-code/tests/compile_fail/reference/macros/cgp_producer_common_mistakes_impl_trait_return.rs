//! `docs/reference/macros/cgp_producer.md`, *Common Mistakes*: `impl Trait` in the return type is
//! rejected.

use cgp::prelude::*;

#[cgp_producer]
fn magic_number() -> impl core::fmt::Display {
    42
}

fn main() {}
