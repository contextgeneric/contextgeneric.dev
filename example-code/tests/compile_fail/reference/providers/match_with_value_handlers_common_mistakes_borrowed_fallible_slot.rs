use cgp::core::error::ErrorTypeProviderComponent;
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
impl<'a, Context, Code> Computer<Context, Code, &'a Circle> for ComputeAreaOfRef {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, circle: &'a Circle) -> f64 {
        core::f64::consts::PI * circle.radius * circle.radius
    }
}

#[cgp_provider]
impl<'a, Context, Code> Computer<Context, Code, &'a Rectangle> for ComputeAreaOfRef {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, rectangle: &'a Rectangle) -> f64 {
        rectangle.width * rectangle.height
    }
}

pub struct App;

// The table routes `HandlerComponent`, but the borrowed matcher behind it has no `Handler` impl.
delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        HandlerComponent: MatchWithValueHandlersRef<ComputeAreaOfRef>,
    }
}

check_components! {
    App {
        HandlerComponent: <'a> ((), &'a Shape),
    }
}

fn main() {}
