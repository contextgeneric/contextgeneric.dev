use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet { fn greet(&self) -> String; }

#[cgp_impl(new GreetHello)]
impl Greeter { fn greet(&self) -> String { "Hello!".to_owned() } }

#[cgp_impl(new GreetQuietly)]
impl Greeter { fn greet(&self) -> String { "hello".to_owned() } }

cgp_namespace! {
    new BaseDefaults {
        GreeterComponent: GreetHello,
    }
}

// error[E0119]: conflicting implementations of trait `QuietDefaults<_>`
//              for type `GreeterComponent`
cgp_namespace! {
    new QuietDefaults: BaseDefaults {
        GreeterComponent: GreetQuietly,
    }
}

fn main() {}
