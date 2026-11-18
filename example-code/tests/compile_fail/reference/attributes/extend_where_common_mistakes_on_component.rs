use cgp::prelude::*;

// error: cannot find attribute `extend_where` in this scope
#[cgp_component(Scaler)]
#[extend_where(Scalar: Clone)]
pub trait CanScale<Scalar> {
    fn scale(&self) -> Scalar;
}

fn main() {}
