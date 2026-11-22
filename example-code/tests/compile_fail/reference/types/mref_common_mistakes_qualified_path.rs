// The getter macros recognize `MRef<'_, T>` by its shape, a single-segment path named `MRef`. A
// qualified path is treated as an owned field type instead, so `'_` lands in a `HasField` bound,
// where it is not allowed. This is the `MRef` page's *Common mistakes* entry on a qualified path.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasGreeting {
    fn greeting(&self) -> cgp::core::field::types::MRef<'_, String>;
}

fn main() {}
