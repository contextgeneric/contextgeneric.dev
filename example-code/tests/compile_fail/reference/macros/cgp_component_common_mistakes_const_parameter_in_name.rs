//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: a const parameter in the `name:`
//! list parses, then fails where the name is used as a type.

use cgp::prelude::*;

#[cgp_component { provider: Buffer, name: BufferComponent<const N: usize> }]
pub trait CanBuffer {
    fn buffer(&self);
}

fn main() {}
