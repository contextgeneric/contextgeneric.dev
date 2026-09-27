//! `docs/reference/macros/cgp_type.md`, *Common Mistakes*: the trait must contain exactly one
//! associated type and nothing else.

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;

    fn zero(&self) -> Self::Scalar;
}

fn main() {}
