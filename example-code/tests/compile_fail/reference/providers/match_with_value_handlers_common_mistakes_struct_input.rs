use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
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

// A struct, not an enum: its field list is a product, which the matcher cannot walk.
#[derive(CgpData)]
pub struct Scene {
    pub circle: Circle,
    pub rectangle: Rectangle,
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: MatchWithValueHandlers<ComputeArea>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), Scene),
    }
}

fn main() {}
