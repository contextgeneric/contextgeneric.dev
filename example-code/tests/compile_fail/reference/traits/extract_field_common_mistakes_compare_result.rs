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

#[derive(Debug, PartialEq, ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn main() {
    let shape = Shape::Circle(Circle { radius: 1.0 });
    // The `Err` half is a generated remainder, which derives nothing.
    assert_eq!(
        shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>),
        Ok(Circle { radius: 1.0 })
    );
}
