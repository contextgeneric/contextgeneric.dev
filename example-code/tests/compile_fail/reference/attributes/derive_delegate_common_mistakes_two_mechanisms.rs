use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
#[derive_delegate(UseDelegate<Shape>)]
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

pub struct App;

// error[E0119]: conflicting implementations of trait `DelegateComponent<AreaCalculatorComponent>`
//               for type `App`
delegate_components! {
    App {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: RectangleArea,

        AreaCalculatorComponent:
            UseDelegate<new AreaCalculatorComponents {
                Rectangle: RectangleArea,
            }>,
    }
}

fn main() {}
