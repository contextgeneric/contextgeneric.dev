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

// The derive already emits this impl.
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;

    fn from_variant(_tag: PhantomData<Symbol!("Circle")>, value: Circle) -> Self {
        Shape::Circle(value)
    }
}

fn main() {}
