//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: `#[uses]` above a component trait
//! is rejected by name.

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
