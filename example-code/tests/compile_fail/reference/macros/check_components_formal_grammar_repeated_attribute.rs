use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) {
        println!("Hello, {name}!");
    }
}

pub struct Person;

// error: Multiple `#[check_trait]` attributes found. Expected at most one.
check_components! {
    #[check_trait(CheckPerson)]
    #[check_trait(CheckPersonAgain)]
    Person {
        GreeterComponent,
    }
}

fn main() {}
