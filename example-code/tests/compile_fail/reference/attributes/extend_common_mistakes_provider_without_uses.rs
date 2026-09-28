use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
#[extend(HasName)]
pub trait CanGreet {
    fn greet(&self);
}

// error[E0277]: the trait bound `__Context__: HasName` is not satisfied
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

fn main() {}
