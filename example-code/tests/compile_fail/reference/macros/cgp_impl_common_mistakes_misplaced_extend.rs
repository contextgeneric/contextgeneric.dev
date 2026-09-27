//! `docs/reference/macros/cgp_impl.md`, *Common Mistakes*: `#[extend]` is not read by `#[cgp_impl]`,
//! so it reaches the compiler as an unresolved attribute.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
#[extend(HasName)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello".to_owned()
    }
}

fn main() {}
