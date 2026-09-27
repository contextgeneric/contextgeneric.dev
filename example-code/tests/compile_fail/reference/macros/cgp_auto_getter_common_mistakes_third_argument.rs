//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: a getter takes at most `self` and one `PhantomData`.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self, index: usize, count: usize) -> &str;
}

fn main() {}
