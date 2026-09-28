use cgp::core::component::UseDefault;
use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str {
        "John"
    }
}

#[cgp_component(Greeter)]
#[extend(HasName)]
pub trait CanGreet {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

#[cgp_impl(UseDefault)]
impl Greeter {}

fn main() {}
