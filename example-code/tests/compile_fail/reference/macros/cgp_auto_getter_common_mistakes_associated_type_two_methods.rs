//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: an associated type allows exactly one getter method.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    type Name;

    fn name(&self) -> &Self::Name;
    fn nickname(&self) -> &Self::Name;
}

fn main() {}
