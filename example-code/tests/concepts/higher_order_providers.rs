//! Code from `docs/concepts/higher-order-providers.md` — *Higher-order providers*.

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

#[cgp_impl(new CircleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] radius: f64) -> f64 {
        core::f64::consts::PI * radius * radius
    }
}

/// ## The bound the inner provider carries
///
/// Written out, so the extra `<Self>` is visible before the attribute hides it.
pub mod the_bound_written_out {
    use cgp::prelude::*;

    use super::{AreaCalculator, AreaCalculatorComponent, RectangleArea};

    #[cgp_impl(new ScaledArea<Inner>)]
    impl<Inner> AreaCalculator
    where
        Inner: AreaCalculator<Self>,
    {
        fn area(&self, #[implicit] scale: f64) -> f64 {
            Inner::area(self) * scale * scale
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub width: f64,
        pub height: f64,
        pub scale: f64,
    }

    delegate_components! {
        App {
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { AreaCalculatorComponent } }
    }
}

/// ## The same provider with the bound completed for you
pub mod the_bound_completed_for_you {
    use cgp::prelude::*;

    use super::{AreaCalculator, AreaCalculatorComponent, CircleArea, RectangleArea};

    #[cgp_impl(new ScaledArea<Inner>)]
    #[use_provider(Inner: AreaCalculator)]
    impl<Inner> AreaCalculator {
        fn area(&self, #[implicit] scale: f64) -> f64 {
            let base = Inner::area(self);
            base * scale * scale
        }
    }

    #[derive(HasField)]
    pub struct ScaledRectangle {
        pub width: f64,
        pub height: f64,
        pub scale: f64,
    }

    #[derive(HasField)]
    pub struct ScaledCircle {
        pub radius: f64,
        pub scale: f64,
    }

    // The same wrapper over two different base cases.
    delegate_components! {
        ScaledRectangle {
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    delegate_components! {
        ScaledCircle {
            AreaCalculatorComponent: ScaledArea<CircleArea>,
        }
    }

    mod check_rectangle {
        use super::*;
        check_components! { ScaledRectangle { AreaCalculatorComponent } }
    }

    mod check_circle {
        use super::*;
        check_components! { ScaledCircle { AreaCalculatorComponent } }
    }

    #[test]
    fn one_wrapper_over_two_base_cases() {
        use crate::concepts::higher_order_providers::CanCalculateArea;

        let rectangle = ScaledRectangle {
            width: 3.0,
            height: 4.0,
            scale: 2.0,
        };
        assert_eq!(rectangle.area(), 48.0);

        let circle = ScaledCircle {
            radius: 1.0,
            scale: 3.0,
        };
        assert!((circle.area() - core::f64::consts::PI * 9.0).abs() < 1e-9);
    }
}

/// ## Composing without naming a context
///
/// Two wrappers stacked, and the whole stack given a name. The alias carries no bounds at all: the
/// requirements are discharged where the context wires it, not where the composition is written.
pub mod composing_without_naming_a_context {
    use cgp::prelude::*;

    use super::the_bound_completed_for_you::ScaledArea;
    use super::{AreaCalculatorComponent, RectangleArea};

    pub type ScaledScaledRectangleArea = ScaledArea<ScaledArea<RectangleArea>>;

    #[derive(HasField)]
    pub struct App {
        pub width: f64,
        pub height: f64,
        pub scale: f64,
    }

    delegate_components! {
        App {
            AreaCalculatorComponent: ScaledScaledRectangleArea,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { AreaCalculatorComponent } }
    }

    #[test]
    fn the_stack_composes_in_types() {
        use crate::concepts::higher_order_providers::CanCalculateArea;

        let app = App {
            width: 3.0,
            height: 4.0,
            scale: 2.0,
        };
        // 12 scaled twice: 12 * 4 * 4
        assert_eq!(app.area(), 192.0);
    }
}

/// ## Falling back to whatever the context already wired
///
/// A provider declared with an explicit struct can default its inner parameter to `UseContext`,
/// which routes the inner step back through the context's own table.
pub mod falling_back_to_the_context {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    #[cgp_component(ShapeAreaCalculator)]
    pub trait CanCalculateShapeArea<Shape> {
        fn shape_area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_impl(new RectangleArea)]
    impl ShapeAreaCalculator<Rectangle> {
        fn shape_area(&self, shape: &Rectangle) -> f64 {
            shape.width * shape.height
        }
    }

    // The default is what makes the unparameterized `SumAreas` mean `SumAreas<UseContext>`, and it
    // needs an explicit struct declaration to be written down.
    pub struct SumAreas<Inner = UseContext>(pub PhantomData<Inner>);

    #[cgp_impl(SumAreas<Inner>)]
    #[use_provider(Inner: ShapeAreaCalculator<Shape>)]
    impl<Shape, Inner> ShapeAreaCalculator<Vec<Shape>> {
        fn shape_area(&self, shapes: &Vec<Shape>) -> f64 {
            shapes.iter().map(|shape| Inner::shape_area(self, shape)).sum()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ShapeAreaCalculatorComponent;

            @ShapeAreaCalculatorComponent.Rectangle: RectangleArea,
            // No inner provider named, so the per-element lookup goes back through this table and
            // finds the `Rectangle` entry above.
            @ShapeAreaCalculatorComponent.<Shape> Vec<Shape>: SumAreas,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ShapeAreaCalculatorComponent: [Rectangle, Vec<Rectangle>],
            }
        }
    }

    #[test]
    fn the_inner_step_resolves_through_the_context() {
        let shapes = vec![
            Rectangle {
                width: 3.0,
                height: 4.0,
            },
            Rectangle {
                width: 1.0,
                height: 2.0,
            },
        ];
        assert_eq!(App.shape_area(&shapes), 14.0);
    }
}

/// ## Not every generic provider is a higher-order one
///
/// `GetName<Tag>` is generic, and its parameter is a field name rather than a provider — nothing is
/// delegated to, so none of the machinery above applies.
pub mod not_every_generic_provider {
    use cgp::prelude::*;

    #[cgp_component(NameGetter)]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_impl(new GetName<Tag>)]
    impl<Tag> NameGetter
    where
        Self: HasField<Tag, Value = String>,
    {
        fn name(&self) -> &str {
            self.get_field(PhantomData::<Tag>)
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub first_name: String,
    }

    delegate_components! {
        App {
            NameGetterComponent: GetName<Symbol!("first_name")>,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { NameGetterComponent } }
    }

    #[test]
    fn the_parameter_is_a_field_name() {
        let app = App {
            first_name: "Ada".to_owned(),
        };
        assert_eq!(app.name(), "Ada");
    }
}
