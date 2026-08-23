//! Code from `docs/reference/providers/dispatch/extract_field_and_handle.md` — `ExtractFieldAndHandle`.

/// ## Usage and Examples
///
/// `ExtractFieldAndHandle<Tag, Provider>` is the per-variant adapter a matcher's list is built from. It
/// tries to extract the variant named `Tag`; on success it hands the payload, still tagged, to
/// `Provider`. Here each variant of `Shape` gets its own adapter, wrapped in `HandleFieldValue` so the
/// handler receives the bare payload.
pub mod one_adapter_per_variant {
    use core::marker::PhantomData;

    use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};
    use cgp::extra::handler::{Computer, ComputerComponent};
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    pub struct Circle {
        pub radius: f64,
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, Circle> for ComputeArea {
        type Output = f64;

        fn compute(_context: &Context, _code: PhantomData<Code>, circle: Circle) -> f64 {
            core::f64::consts::PI * circle.radius * circle.radius
        }
    }

    #[cgp_provider]
    impl<Context, Code> Computer<Context, Code, Rectangle> for ComputeArea {
        type Output = f64;

        fn compute(_context: &Context, _code: PhantomData<Code>, rectangle: Rectangle) -> f64 {
            rectangle.width * rectangle.height
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: MatchWithHandlers<
                Product![
                    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
                    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
                ]
            >,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: ((), Shape),
            }
        }
    }

    #[test]
    fn the_adapter_extracts_its_variant_and_forwards_the_payload() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(
            app.compute(code, Shape::Rectangle(Rectangle { width: 2.0, height: 5.0 })),
            10.0,
        );
    }
}
