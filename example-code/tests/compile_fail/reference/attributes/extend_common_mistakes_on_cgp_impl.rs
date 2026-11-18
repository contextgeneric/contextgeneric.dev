use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

// error: cannot find attribute `extend` in this scope
// error[E0599]: the method `name` exists for reference `&__Context__`, but its trait bounds were not satisfied
#[cgp_impl(new GreetHello)]
#[extend(HasName)]
impl Greeter {
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

fn main() {}
