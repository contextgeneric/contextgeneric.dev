use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};
use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, Circle> for ComputeArea {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, circle: Circle) -> f64 {
        core::f64::consts::PI * circle.radius * circle.radius
    }
}

#[cgp_provider]
impl<Context, Code> Computer<Context, Code, Rectangle> for ComputeArea {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, rectangle: Rectangle) -> f64 {
        rectangle.width * rectangle.height
    }
}

pub struct App;

// The list has no arm for `Rectangle`, so the remainder is still inhabited.
delegate_components! {
    App {
        ComputerComponent:
            MatchWithHandlers<Product![
                ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
            ]>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), Shape),
    }
}

fn main() {}
