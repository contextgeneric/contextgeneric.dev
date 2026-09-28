use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}

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
    // error[E0283]: type annotations needed
    //
    // The payload type does not select the variant, so the tag must be written.
    let _ = Shape::from_variant(PhantomData, Circle { radius: 2.0 });
}
