use core::fmt::Display;

use cgp::prelude::*;

// error: cannot find attribute `impl_generics` in this scope
//
// Reported once, although every item the component macro generates carries the attribute.
#[cgp_component(Greeter)]
#[impl_generics(Name: Display)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

fn main() {}
