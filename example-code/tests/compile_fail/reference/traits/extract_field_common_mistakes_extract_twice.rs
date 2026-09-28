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

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn main() {
    let shape = Shape::Circle(Circle { radius: 1.0 });
    if let Err(remainder) = shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
        // `Circle` is already ruled out of `remainder`.
        let _ = remainder.extract_field(PhantomData::<Symbol!("Circle")>);
    }
}
