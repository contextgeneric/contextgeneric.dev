//! `docs/reference/macros/cgp_producer.md`, *Common Mistakes*: a producer takes no parameters.

use cgp::prelude::*;

#[cgp_producer]
fn magic_number(seed: u64) -> u64 {
    seed
}

fn main() {}
