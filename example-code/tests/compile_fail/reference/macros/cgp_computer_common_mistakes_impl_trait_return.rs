//! `docs/reference/macros/cgp_computer.md`, *Common Mistakes*: `impl Trait` in the return type is
//! rejected.

use cgp::prelude::*;

#[cgp_computer]
fn describe(value: u64) -> impl core::fmt::Display {
    value
}

fn main() {}
