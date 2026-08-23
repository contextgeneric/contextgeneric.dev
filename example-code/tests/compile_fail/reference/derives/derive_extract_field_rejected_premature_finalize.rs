use cgp::core::field::traits::FinalizeExtractResult;
use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}
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

    // Finalizing after extracting only one of the two variants.
    let _ = shape
        .to_extractor()
        .extract_field(PhantomData::<Symbol!("Circle")>)
        .finalize_extract_result();
}
