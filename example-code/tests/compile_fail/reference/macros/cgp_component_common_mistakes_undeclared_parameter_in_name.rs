//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: a `name:` parameter the trait does
//! not declare passes the parser and fails in the generated code.

use cgp::prelude::*;

#[cgp_component { provider: Shape, name: ShapeComponent<T> }]
pub trait CanShape {
    fn shape(&self);
}

fn main() {}
