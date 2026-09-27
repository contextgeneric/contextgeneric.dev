//! `docs/reference/macros/cgp_computer.md`, *Common Mistakes*: the fallible members need an error
//! type on the context, while `compute` does not.

use core::marker::PhantomData;

use cgp::extra::handler::{Computer, TryComputer};
use cgp::prelude::*;

#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}

pub struct App;

fn main() {
    let _ = Add::compute(&App, PhantomData::<()>, (1, 2));
    let _ = Add::try_compute(&App, PhantomData::<()>, (1, 2));
}
