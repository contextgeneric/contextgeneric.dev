use cgp::prelude::*;

// The component lacks `#[derive_delegate(UseDelegate<Shape>)]`.
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle;

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<Rectangle> {
    fn area(&self, _shape: &Rectangle) -> f64 {
        1.0
    }
}

pub struct MyApp;

delegate_components! {
    MyApp {
        AreaCalculatorComponent:
            UseDelegate<new AreaCalculatorComponents {
                Rectangle: RectangleArea,
            }>,
    }
}

// error[E0277]: `UseDelegate<AreaCalculatorComponents>: IsProviderFor<..>` is not satisfied
check_components! {
    MyApp {
        AreaCalculatorComponent: Rectangle,
    }
}

fn main() {}
