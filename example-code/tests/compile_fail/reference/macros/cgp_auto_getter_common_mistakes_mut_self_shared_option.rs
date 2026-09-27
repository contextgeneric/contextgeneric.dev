//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: a `&mut self` getter returning a shared `Option<&T>` fails with `E0308`.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasMaybe {
    fn maybe(&mut self) -> Option<&u32>;
}

fn main() {}
