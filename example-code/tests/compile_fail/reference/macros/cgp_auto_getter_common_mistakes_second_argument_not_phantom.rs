//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: the second argument must be a `PhantomData`.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self, index: usize) -> &str;
}

fn main() {}
