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
    // The variant is `Circle`, not `circle`.
    let _ = Shape::from_variant(PhantomData::<Symbol!("circle")>, Circle { radius: 1.0 });
}
