//! Code from `docs/reference/providers/dispatch/build_with_handlers.md` — `BuildWithHandlers`.
//!
//! Pins that `BuildWithHandlers` assembles a record from a `Product!` of builder adapters, starting
//! from an empty builder and finalizing the all-present result. Dropping a handler for some field would
//! fail to compile at the finalize step.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::dispatch::{BuildAndMerge, BuildAndSetField, BuildWithHandlers};
    use cgp::extra::handler::ComputerComponent;
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct FooBarBaz {
        pub foo: u64,
        pub bar: String,
        pub baz: bool,
    }

    #[derive(CgpData)]
    pub struct FooBar {
        pub foo: u64,
        pub bar: String,
    }

    #[cgp_producer]
    fn build_foo_bar() -> FooBar {
        FooBar {
            foo: 1,
            bar: "bar".to_owned(),
        }
    }

    #[cgp_producer(BuildBaz)]
    fn build_baz() -> bool {
        true
    }

    pub type Handlers =
        Product![BuildAndMerge<BuildFooBar>, BuildAndSetField<Symbol!("baz"), BuildBaz>];

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: BuildWithHandlers<FooBarBaz, Handlers>,
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
    fn test_build_with_handlers() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(
            app.compute(code, ()),
            FooBarBaz {
                foo: 1,
                bar: "bar".to_owned(),
                baz: true,
            },
        );
    }
}
