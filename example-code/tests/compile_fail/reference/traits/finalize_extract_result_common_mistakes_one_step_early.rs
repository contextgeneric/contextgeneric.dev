use cgp::core::field::traits::FinalizeExtractResult;
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
    let shape = Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 });
    // Only `Rectangle` has been tried; `Circle` is still possible.
    let _rect = shape
        .to_extractor()
        .extract_field(PhantomData::<Symbol!("Rectangle")>)
        .finalize_extract_result();
}
