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

// error: Invalid attribute #[allow(unused)]
check_components! {
    #[allow(unused)]
    Person {
        GreeterComponent,
    }
}

fn main() {}
