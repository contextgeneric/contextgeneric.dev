use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Gen<T> {
    pub width: f64,
    pub height: f64,
    pub marker: core::marker::PhantomData<T>,
}

delegate_components! {
    <T> Gen<T> {
        AreaCalculatorComponent: RectangleArea,
    }
}

// error[E0425]: cannot find type `T` in this scope, then E0207
check_components! {
    #[check_providers(RectangleArea)]
    <T> Gen<T> {
        AreaCalculatorComponent,
    }
}

fn main() {}
