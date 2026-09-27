//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: `#[uses]` above a component trait
//! is carried onto the generated items and reported once as an unresolved attribute.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
#[uses(HasName)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

fn main() {}
