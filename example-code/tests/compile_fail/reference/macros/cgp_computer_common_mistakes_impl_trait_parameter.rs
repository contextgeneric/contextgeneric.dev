//! `docs/reference/macros/cgp_computer.md`, *Common Mistakes*: `impl Trait` in a parameter is
//! rejected; a generic parameter is the fix.

use cgp::prelude::*;

#[cgp_computer]
fn describe(value: impl core::fmt::Display) -> String {
    value.to_string()
}

fn main() {}
