//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: a const generic parameter on the
//! trait is rejected by the macro.

use cgp::prelude::*;

#[cgp_component(BufferProvider)]
pub trait CanBuffer<const N: usize> {
    fn buffer(&self) -> [u8; N];
}

fn main() {}
