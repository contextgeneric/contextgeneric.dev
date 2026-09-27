//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: a getter method must be a plain signature.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    async fn name(&self) -> &str;
}

fn main() {}
