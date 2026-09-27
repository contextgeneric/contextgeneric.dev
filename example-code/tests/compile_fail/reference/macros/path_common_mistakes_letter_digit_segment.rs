use cgp::prelude::*;

// error[E0425]: the segment `f` is read as a primitive-shaped type, not a symbol
pub type Route = Path!(@app.f);

fn main() {}
