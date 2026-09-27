//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: the macro takes no argument.

use cgp::prelude::*;

#[cgp_auto_getter(NameGetter)]
pub trait HasName {
    fn name(&self) -> &str;
}

fn main() {}
