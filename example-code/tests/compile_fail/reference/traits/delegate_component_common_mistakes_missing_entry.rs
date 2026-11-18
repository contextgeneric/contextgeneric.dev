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

// `App` has no table entry for `GreeterComponent`.
check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
