//! Code from `docs/concepts/aggregate-providers.md` — *Aggregate providers*.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(PerimeterCalculator)]
pub trait CanCalculatePerimeter {
    fn perimeter(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[cgp_impl(new RectanglePerimeter)]
impl PerimeterCalculator {
    fn perimeter(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        2.0 * (width + height)
    }
}

/// ## A table whose target is not a context
///
/// The `new` keyword declares the struct and gives it a table. Nothing here is a context.
pub mod a_table_that_is_not_a_context {
    use cgp::prelude::*;

    use super::*;

    delegate_components! {
        new GeometryComponents {
            AreaCalculatorComponent: RectangleArea,
            PerimeterCalculatorComponent: RectanglePerimeter,
        }
    }

    /// ## How a context uses one
    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        Rectangle {
            [
                AreaCalculatorComponent,
                PerimeterCalculatorComponent,
            ]: GeometryComponents,
        }
    }

    // The bundle is verified here — through a real context, which is the only place the question
    // means anything.
    mod check_rectangle {
        use super::*;
        check_components! {
            Rectangle {
                AreaCalculatorComponent,
                PerimeterCalculatorComponent,
            }
        }
    }

    #[test]
    fn the_context_stays_the_context_all_the_way_down() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        // `RectangleArea` reads `width` and `height` from `Rectangle`, not from the bundle.
        assert_eq!(rectangle.area(), 12.0);
        assert_eq!(rectangle.perimeter(), 14.0);
    }
}

/// ## Bundles nest
pub mod bundles_nest {
    use cgp::prelude::*;

    use super::*;

    delegate_components! {
        new BaseGeometry {
            AreaCalculatorComponent: RectangleArea,
            PerimeterCalculatorComponent: RectanglePerimeter,
        }
    }

    // A second bundle that takes everything from the first and overrides nothing yet. The delegate
    // of an entry is allowed to be another table rather than a leaf provider.
    delegate_components! {
        new ExtendedGeometry {
            [
                AreaCalculatorComponent,
                PerimeterCalculatorComponent,
            ]: BaseGeometry,
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        Rectangle {
            [
                AreaCalculatorComponent,
                PerimeterCalculatorComponent,
            ]: ExtendedGeometry,
        }
    }

    mod check_rectangle {
        use super::*;
        check_components! {
            Rectangle {
                AreaCalculatorComponent,
                PerimeterCalculatorComponent,
            }
        }
    }

    #[test]
    fn resolution_walks_each_table_in_turn() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert_eq!(rectangle.area(), 12.0);
    }
}

/// ## Checking a bundle asks the wrong question
///
/// A context-side check on a bundle asks whether the *bundle* can use the component as a context,
/// which is a role it never plays. Which way that goes wrong depends on what is bundled, and one of
/// the two ways is silent.
///
/// The silent one. `AlwaysOne` needs nothing from its context, so it implements the provider trait
/// for every context including the bundle — the check passes and proves nothing. It is in the module
/// body below, since it compiles.
///
/// The loud one. `RectangleArea` needs fields, and the check demands them of the bundle:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/aggregate_providers_checking_a_bundle_asks_the_wrong_question.rs`.
pub mod checking_a_bundle_asks_the_wrong_question {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_impl(new AlwaysOne)]
    impl AreaCalculator {
        fn area(&self) -> f64 {
            1.0
        }
    }

    // The silent pass: `delegate_and_check_components!` on the bundle compiles, because `AlwaysOne`
    // needs nothing from a context, so the check proves nothing.
    delegate_and_check_components! {
        new GeometryComponents {
            AreaCalculatorComponent: AlwaysOne,
        }
    }
}

/// ## Checking the bundle on its own, correctly
///
/// `#[check_providers]` asserts the provider-side question — is this a provider for that context —
/// which is the one the bundle's role calls for.
pub mod checking_the_bundle_correctly {
    use cgp::prelude::*;

    use super::*;

    delegate_components! {
        new GeometryComponents {
            AreaCalculatorComponent: RectangleArea,
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        Rectangle {
            AreaCalculatorComponent: GeometryComponents,
        }
    }

    mod check_the_bundle_for_a_real_context {
        use super::*;
        check_components! {
            #[check_providers(
                RectangleArea,
                GeometryComponents,
            )]
            Rectangle {
                AreaCalculatorComponent,
            }
        }
    }
}
