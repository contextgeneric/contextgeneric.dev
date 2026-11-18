//! `docs/tutorials/area-calculation/checking.md` — the mis-wired program the page starts from.
//!
//! `PlainCircle` is wired to `RectangleAreaCalculator`, which needs `width` and `height` that a
//! circle does not have. The page's point is that this compiles until something checks it, so the
//! `check_components!` below is what turns it into the error the page quotes.

use cgp::prelude::*;
use core::f64::consts::PI;

#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}

#[cgp_fn]
pub fn circle_area(&self, #[implicit] radius: f64) -> f64 {
    PI * radius * radius
}

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleAreaCalculator)]
#[uses(RectangleArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        self.rectangle_area()
    }
}

#[cgp_impl(new CircleAreaCalculator)]
#[uses(CircleArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        self.circle_area()
    }
}

#[derive(HasField)]
pub struct PlainRectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(HasField)]
pub struct PlainCircle {
    pub radius: f64,
}

delegate_components! {
    PlainRectangle {
        AreaCalculatorComponent: RectangleAreaCalculator,
    }
}

delegate_components! {
    PlainCircle {
        AreaCalculatorComponent: RectangleAreaCalculator,
    }
}

check_components! {
    PlainCircle {
        AreaCalculatorComponent,
    }
}

fn main() {}
