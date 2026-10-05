// From docs/cargo-cgp/reading-output.md, Two entries that claim one key.

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

#[cgp_impl(new GreetGoodbye)]
impl Greeter {
    fn greet(&self) -> String {
        "Goodbye!".to_owned()
    }
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: GreetHello,
        GreeterComponent: GreetGoodbye,
    }
}

fn main() {}
