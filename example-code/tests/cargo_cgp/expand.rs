//! Code from `docs/cargo-cgp/expand.md` — *Expand*.
//!
//! The same program as [`super::check`] with its missing `height` field restored, which is the state
//! the page expands. The listing the page shows is *generated* code, so this file can only confirm the
//! program it was generated from still compiles and wires up; the authority on the listing's text is
//! `cargo cgp expand --lib --item Rectangle`, which is how it was produced.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wiring_resolves() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert_eq!(rectangle.area(), 12.0);
    }
}
