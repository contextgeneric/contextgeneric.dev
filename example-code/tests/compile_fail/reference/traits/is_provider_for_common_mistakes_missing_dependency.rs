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
#[uses(HasName)]
impl Greeter {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

// The field is `first_name`, not `name`.
#[derive(HasField)]
pub struct Person {
    pub first_name: String,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    Person {
        GreeterComponent,
    }
}

fn main() {}
