use cgp::prelude::*;

cgp_namespace! {
    new AppNamespace {}
}

// error[E0119]: conflicting implementations of trait `AppNamespace<_>` for type `GreeterComponent`
#[cgp_component(Greeter)]
#[prefix(@app in AppNamespace)]
#[prefix(@other in AppNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

fn main() {}
