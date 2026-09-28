use core::fmt::Display;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

// error: cannot find attribute `impl_generics` in this scope
// error[E0425]: cannot find type `Name` in this scope
#[cgp_impl(new GreetHello)]
#[impl_generics(Name: Display)]
impl Greeter {
    fn greet(&self, #[implicit] name: &Name) -> String {
        format!("Hello, {name}!")
    }
}

fn main() {}
