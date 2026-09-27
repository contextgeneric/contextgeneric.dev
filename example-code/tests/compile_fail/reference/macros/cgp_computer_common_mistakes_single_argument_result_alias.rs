//! `docs/reference/macros/cgp_computer.md`, *Common Mistakes*: a one-argument `Result<T>` alias fails
//! to parse, because the macro expects `Result<T, E>`.

use cgp::prelude::*;

pub type Result<T> = core::result::Result<T, String>;

#[cgp_computer]
fn halve(value: u64) -> Result<u64> {
    Ok(value / 2)
}

fn main() {}
