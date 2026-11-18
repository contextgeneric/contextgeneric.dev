use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello!".to_owned()
    }
}

#[cgp_impl(new GreetHi)]
impl Greeter {
    fn greet(&self) -> String {
        "Hi!".to_owned()
    }
}

pub struct App;

// Two entries for one key are two impls of one trait for one type.
delegate_components! {
    App {
        GreeterComponent: GreetHello,
        GreeterComponent: GreetHi,
    }
}

fn main() {}
