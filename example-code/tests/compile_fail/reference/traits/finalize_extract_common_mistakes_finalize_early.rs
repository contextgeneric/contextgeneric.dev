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
    let _: f64 = match shape.to_extractor().extract_field(PhantomData::<Symbol!("Rectangle")>) {
        Ok(rect) => rect.width,
        // `Circle` is still possible, so the remainder is inhabited.
        Err(remainder) => remainder.finalize_extract(),
    };
}
