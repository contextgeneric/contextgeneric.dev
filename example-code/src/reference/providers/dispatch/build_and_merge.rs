//! Code from `docs/reference/providers/dispatch/build_and_merge.md` — `BuildAndMerge`.
//!
//! Pins that `BuildAndMerge<Provider>` copies a whole sub-record's shared fields into the builder at
//! once, beside a single-field `BuildAndSetField`, under `BuildWithHandlers`.

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

    // `BuildAndMerge<BuildFooBar>` copies `foo` and `bar`; `BuildAndSetField` sets `baz`.
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
    fn test_build_and_merge() {
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
