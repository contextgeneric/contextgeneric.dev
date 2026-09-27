//! `docs/reference/macros/cgp_getter.md`, *Common Mistakes*: a multi-method trait gets no `UseField` provider.

use cgp::prelude::*;

#[cgp_getter]
pub trait HasDimensions {
    fn width(&self) -> &f64;
    fn height(&self) -> &f64;
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        DimensionsGetterComponent: UseField<Symbol!("width")>,
    }
}

check_components! {
    Rectangle {
        DimensionsGetterComponent,
    }
}

fn main() {}
