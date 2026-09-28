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
}

fn main() {
    let shape = Shape::Circle(Circle { radius: 1.0 });
    // `FinalizeExtractResult` is not in the prelude, and it is not imported here.
    let _circle = shape
        .to_extractor()
        .extract_field(PhantomData::<Symbol!("Circle")>)
        .finalize_extract_result();
}
