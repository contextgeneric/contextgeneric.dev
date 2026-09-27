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

// error: expected identifier, at the context type
check_components! {
    <'a> &'a Person {
        GreeterComponent,
    }
}

fn main() {}
