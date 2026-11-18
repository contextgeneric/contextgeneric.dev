use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

cgp_namespace! {
    new AppNamespace: DefaultNamespace {}
}

// error[E0119]: conflicting implementations of trait `AppNamespace<_>` for type `GreeterComponent`
#[cgp_impl(new GreetHello)]
#[default_impl(GreeterComponent in AppNamespace)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello!".to_owned()
    }
}

fn main() {}
