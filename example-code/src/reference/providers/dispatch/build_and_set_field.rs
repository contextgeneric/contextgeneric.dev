//! Code from `docs/reference/providers/dispatch/build_and_set_field.md` — `BuildAndSetField`.
//!
//! Pins that `BuildAndSetField<Tag, Provider>` computes one named field and sets it on the builder, and
//! that a `Product!` of them assembles a whole record through `BuildWithHandlers`.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::dispatch::{BuildAndSetField, BuildWithHandlers};
    use cgp::extra::handler::ComputerComponent;
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct Rectangle {
        pub width: u64,
        pub height: u64,
    }

    #[cgp_producer(ComputeWidth)]
    fn compute_width() -> u64 {
        3
    }

    #[cgp_producer(ComputeHeight)]
    fn compute_height() -> u64 {
        4
    }

    pub type Handlers = Product![
        BuildAndSetField<Symbol!("width"), ComputeWidth>,
        BuildAndSetField<Symbol!("height"), ComputeHeight>,
    ];

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: BuildWithHandlers<Rectangle, Handlers>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: ((), ()),
            }
        }
    }

    #[test]
    fn test_build_and_set_field() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(
            app.compute(code, ()),
            Rectangle {
                width: 3,
                height: 4,
            },
        );
    }
}
