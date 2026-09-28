use cgp::prelude::*;

// error[E0425]: cannot find type `Shape` in this scope
#[cgp_component(Greeter)]
#[derive_delegate(UseDelegate<Shape>)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

fn main() {}
