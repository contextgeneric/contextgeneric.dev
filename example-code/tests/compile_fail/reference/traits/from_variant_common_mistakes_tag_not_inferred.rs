use cgp::prelude::*;

#[derive(Debug, PartialEq)]
pub struct Circle {
    pub radius: f64,
}

#[derive(Debug, PartialEq)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(FromVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn main() {
    // Two variants, so the tag cannot be read off the payload.
    let _ = Shape::from_variant(PhantomData, Circle { radius: 1.0 });
}
