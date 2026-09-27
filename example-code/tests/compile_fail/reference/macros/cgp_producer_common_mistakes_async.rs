//! `docs/reference/macros/cgp_producer.md`, *Common Mistakes*: a producer cannot be `async`.

use cgp::prelude::*;

#[cgp_producer]
async fn magic_number() -> u64 {
    42
}

fn main() {}
