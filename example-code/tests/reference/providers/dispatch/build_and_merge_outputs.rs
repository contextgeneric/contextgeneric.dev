//! Code from `docs/reference/providers/dispatch/build_and_merge_outputs.md` — `BuildAndMergeOutputs`.
//!
//! Pins that `BuildAndMergeOutputs<Output, Handlers>` wraps each plain field-producing provider in a
//! merge step, so a list of sub-record producers assembles the whole record without the caller writing
//! the merges.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::dispatch::BuildAndMergeOutputs;
    use cgp::extra::handler::ComputerComponent;
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct App {
        pub foo: u64,
        pub bar: String,
        pub baz: bool,
    }

    #[derive(CgpData)]
    pub struct FooBar {
        pub foo: u64,
        pub bar: String,
    }

    #[derive(CgpData)]
    pub struct BazPart {
        pub baz: bool,
    }

    #[cgp_producer]
    fn build_foo_bar() -> FooBar {
        FooBar {
            foo: 1,
            bar: "bar".to_owned(),
        }
    }

    #[cgp_producer]
    fn build_baz_part() -> BazPart {
        BazPart { baz: true }
    }

    pub struct AppBuilder;

    delegate_components! {
        AppBuilder {
            ComputerComponent:
                BuildAndMergeOutputs<App, Product![BuildFooBar, BuildBazPart]>,
        }
    }

    mod check_builder {
        use super::*;
        check_components! {
            AppBuilder {
                ComputerComponent: ((), ()),
            }
        }
    }

    #[test]
    fn test_build_and_merge_outputs() {
        use cgp::extra::handler::CanCompute;

        let builder = AppBuilder;
        let code = PhantomData::<()>;

        assert_eq!(
            builder.compute(code, ()),
            App {
                foo: 1,
                bar: "bar".to_owned(),
                baz: true,
            },
        );
    }
}
