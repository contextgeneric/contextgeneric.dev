//! `docs/reference/macros/cgp_producer.md`, *Common Mistakes*: a producer takes no generic parameters.

use cgp::prelude::*;

#[cgp_producer]
fn default_value<T: Default>() -> T {
    T::default()
}

fn main() {}
