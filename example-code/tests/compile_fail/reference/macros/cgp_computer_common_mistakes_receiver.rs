//! `docs/reference/macros/cgp_computer.md`, *Common mistakes*: a computer function takes no receiver.

use cgp::prelude::*;

pub struct Adder;

impl Adder {
    #[cgp_computer]
    fn add(&self, a: u64, b: u64) -> u64 {
        a + b
    }
}

fn main() {}
