//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: `Option<&[T]>` has no rule and reaches the compiler as an unsized bound.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasBytes {
    fn bytes(&self) -> Option<&[u8]>;
}

fn main() {}
