use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(PerimeterCalculator)]
pub trait CanCalculatePerimeter {
    fn perimeter(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        1.0
    }
}

pub struct App;

pub struct WidthKey<T>(pub core::marker::PhantomData<T>);

// error: invalid impl generics syntax
delegate_components! {
    App {
        <T = u32> WidthKey<T>: RectangleArea,
    }
}

fn main() {}
