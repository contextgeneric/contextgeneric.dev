use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".into()
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        GreeterComponent: GreetHello,
    }
}

fn main() {}
