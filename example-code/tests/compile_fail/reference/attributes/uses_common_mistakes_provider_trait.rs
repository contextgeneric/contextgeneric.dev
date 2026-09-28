use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

// error[E0107]: missing generics for trait `AreaCalculator`
#[cgp_fn]
#[uses(AreaCalculator)]
pub fn scaled_area(&self, #[implicit] scale_factor: f64) -> f64 {
    self.area() * scale_factor * scale_factor
}

fn main() {}
