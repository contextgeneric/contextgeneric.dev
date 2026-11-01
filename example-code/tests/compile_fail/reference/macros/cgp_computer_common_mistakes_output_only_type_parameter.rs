//! `docs/reference/macros/cgp_computer.md`, *Common Mistakes*: a type parameter used only in the
//! return type is rejected, since the generated impl cannot determine it.

use cgp::prelude::*;

#[cgp_computer]
fn parse<T: core::str::FromStr>(value: String) -> Option<T> {
    value.parse().ok()
}

fn main() {}
