//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: the macro rejects a `name:`
//! parameter the trait does not declare.

use cgp::prelude::*;

#[cgp_component { provider: Shape, name: ShapeComponent<T> }]
pub trait CanShape {
    fn shape(&self);
}

fn main() {}
