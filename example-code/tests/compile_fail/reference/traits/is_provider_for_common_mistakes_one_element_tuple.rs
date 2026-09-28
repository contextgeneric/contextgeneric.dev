use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<Rectangle> {
    fn area(&self, rectangle: &Rectangle) -> f64 {
        rectangle.width * rectangle.height
    }
}

pub struct App;

// One parameter is passed directly, not as a one-element tuple.
fn assert_provider()
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, App, (Rectangle,)>,
{
}

fn main() {
    assert_provider();
}
