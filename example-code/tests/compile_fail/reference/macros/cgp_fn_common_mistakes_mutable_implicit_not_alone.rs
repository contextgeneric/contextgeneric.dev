//! `docs/reference/macros/cgp_fn.md`, *Common Mistakes*: a mutable implicit argument must be the
//! only implicit argument.

use cgp::prelude::*;

#[cgp_fn]
pub fn grow(&mut self, #[implicit] width: &mut f64, #[implicit] height: f64) {
    *width *= height;
}

fn main() {}
