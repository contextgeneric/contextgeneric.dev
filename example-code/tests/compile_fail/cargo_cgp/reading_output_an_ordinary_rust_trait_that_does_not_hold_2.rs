// From docs/cargo-cgp/reading-output.md, An ordinary Rust trait that does not hold: at a method call.

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[cgp_component(ScalarEquality)]
#[use_type(HasScalarType.Scalar)]
pub trait CanCompareScalars {
    fn same(&self, a: &Scalar, b: &Scalar) -> bool;
}

#[cgp_impl(new CompareScalars)]
#[use_type(HasScalarType.Scalar)]
impl ScalarEquality
where
    Scalar: Eq,
{
    fn same(&self, a: &Scalar, b: &Scalar) -> bool {
        a == b
    }
}

pub struct App;

delegate_components! {
    App {
        ScalarTypeProviderComponent: UseType<f64>,
        ScalarEqualityComponent: CompareScalars,
    }
}

pub fn compare() -> bool {
    App.same(&1.0, &2.0)
}

fn main() {}
