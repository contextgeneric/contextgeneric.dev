use cgp::prelude::*;

// error: expect non-empty tuple list of identifiers in use_delegate_spec
#[cgp_component(AreaCalculator)]
#[derive_delegate(UseDelegate<()>)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

fn main() {}
