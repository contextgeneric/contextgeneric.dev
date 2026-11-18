//! `docs/reference/macros/cgp_type.md`, *Common Mistakes*: the default name comes from the
//! associated type, so a key guessed from the trait name does not exist.

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

pub struct App;

delegate_components! {
    App {
        HasScalarTypeComponent: UseType<f64>,
    }
}

fn main() {}
