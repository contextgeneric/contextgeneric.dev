use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

// error: cannot find attribute `prefix` in this scope
#[cgp_impl(new GreetHello)]
#[prefix(@app in DefaultNamespace)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello!".to_owned()
    }
}

fn main() {}
