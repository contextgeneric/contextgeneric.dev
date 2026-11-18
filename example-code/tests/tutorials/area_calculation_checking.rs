//! `docs/tutorials/area-calculation/checking.md`
//!
//! The page walks one mistake through three diagnostics and then fixes it. What is checked here is
//! the fixed program, plus the hand-written equivalent of `check_components!` that the page's
//! "How it works" section claims is close to what the macro generates. The mis-wired program the
//! page starts from is a compile-fail fixture, at
//! `tests/compile_fail/tutorials/area_calculation_checking_miswired.rs`.

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
        AreaCalculatorComponent: CircleAreaCalculator,
    }
}

check_components! {
    PlainRectangle {
        AreaCalculatorComponent,
    }
}

check_components! {
    PlainCircle {
        AreaCalculatorComponent,
    }
}

// The page's "How it works" section: a check is an ordinary trait bound asserted on purpose. This
// compiles only because the wiring above is correct, which is the whole of what it demonstrates.
pub trait CanUseAreaCalculator: CanCalculateArea {}

impl CanUseAreaCalculator for PlainCircle {}

#[test]
fn the_fixed_wiring_computes_a_circle_area() {
    let circle = PlainCircle { radius: 2.0 };

    assert_eq!(circle.area(), PI * 4.0);
}

#[test]
fn the_rectangle_still_works() {
    let rectangle = PlainRectangle {
        width: 3.0,
        height: 4.0,
    };

    assert_eq!(rectangle.area(), 12.0);
}
