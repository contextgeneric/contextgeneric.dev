//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: a const parameter in the `name:`
//! list parses, then is rejected as a parameter the trait does not declare.

use cgp::prelude::*;

#[cgp_component { provider: Buffer, name: BufferComponent<const N: usize> }]
pub trait CanBuffer {
    fn buffer(&self);
}

fn main() {}
