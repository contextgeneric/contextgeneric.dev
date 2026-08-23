use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) {
        let _ = self.name();
    }
}

#[derive(HasField)]
pub struct Person {
    pub age: u8,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

// error[E0277]: [CGP-E001] the consumer trait `CanGreet` is not implemented for
//              context `Person`
//   root cause: [CGP-E106] missing field `name` on `Person`
check_components! {
    Person {
        GreeterComponent,
    }
}

fn main() {}
