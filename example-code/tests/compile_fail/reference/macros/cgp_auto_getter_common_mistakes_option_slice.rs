//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: the macro rejects `Option<&[T]>`, since the field it would read cannot exist.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasBytes {
    fn bytes(&self) -> Option<&[u8]>;
}

fn main() {}
