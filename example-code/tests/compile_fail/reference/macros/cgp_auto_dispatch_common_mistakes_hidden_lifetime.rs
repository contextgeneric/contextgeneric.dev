//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: a lifetime hidden in a type's
//! path, as in `Cow<str>`, gives the macro nothing to name, so the argument fails with `E0726` and
//! the return type with `E0106`.

use std::borrow::Cow;

use cgp::prelude::*;

#[cgp_auto_dispatch]
pub trait HasLabel {
    fn measure(&self, label: Cow<str>) -> usize;

    fn label(&self) -> Cow<str>;
}

fn main() {}
