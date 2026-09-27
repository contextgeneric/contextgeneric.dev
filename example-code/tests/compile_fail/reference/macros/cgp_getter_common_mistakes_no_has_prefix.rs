//! `docs/reference/macros/cgp_getter.md`, *Common Mistakes*: a trait not named `Has…` gets no default provider name.

use cgp::prelude::*;

#[cgp_getter]
pub trait Dimensions {
    fn width(&self) -> &f64;
}

fn main() {}
