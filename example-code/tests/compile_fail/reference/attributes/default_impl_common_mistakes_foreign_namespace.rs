use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

// error[E0210]: type parameter `__Components__` must be used as an argument to some local type
#[cgp_impl(new GreetHello)]
#[default_impl(@app.GreeterComponent in DefaultNamespace)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello!".to_owned()
    }
}

fn main() {}
