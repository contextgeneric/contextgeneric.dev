//! `docs/reference/macros/cgp_type.md`, *Common Mistakes*: a bound on the type is enforced at the
//! check, not at the wiring line.

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar: Copy;
}

pub struct App;

delegate_components! {
    App {
        ScalarTypeProviderComponent: UseType<String>,
    }
}

check_components! {
    App {
        ScalarTypeProviderComponent,
    }
}

fn main() {}
