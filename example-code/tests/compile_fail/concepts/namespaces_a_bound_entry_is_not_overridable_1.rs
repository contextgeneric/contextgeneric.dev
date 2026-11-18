use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet { fn greet(&self) -> String; }

#[cgp_impl(new GreetHello)]
impl Greeter { fn greet(&self) -> String { "Hello!".to_owned() } }

#[cgp_impl(new GreetQuietly)]
impl Greeter { fn greet(&self) -> String { "hello".to_owned() } }

cgp_namespace! {
    new AppDefaults {
        GreeterComponent: GreetHello,
    }
}

pub struct TestApp;

// error[E0119]: conflicting implementations of trait
//              `DelegateComponent<GreeterComponent>` for type `TestApp`
delegate_components! {
    TestApp {
        namespace AppDefaults;

        GreeterComponent: GreetQuietly,
    }
}

fn main() {}
