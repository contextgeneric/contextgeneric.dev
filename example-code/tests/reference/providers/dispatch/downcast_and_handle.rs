//! Code from `docs/reference/providers/dispatch/downcast_and_handle.md` — `DowncastAndHandle`.
//!
//! Pins that `DowncastAndHandle<Inner, Provider>` matches a group of variants at once by narrowing to a
//! smaller enum, beside a single-variant `ExtractFieldAndHandle`, inside one `MatchWithHandlers` list.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::dispatch::{
        DowncastAndHandle, ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers,
    };
    use cgp::extra::handler::ComputerComponent;
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum FooBarBaz {
        Foo(u64),
        Bar(String),
        Baz(bool),
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum FooBar {
        Foo(u64),
        Bar(String),
    }

    #[cgp_computer]
    fn show_foo_bar(input: FooBar) -> String {
        format!("FooBar::{input:?}")
    }

    #[cgp_computer]
    fn show_baz(input: bool) -> String {
        format!("Baz({input:?})")
    }

    // `Foo` and `Bar` are handled together by narrowing to `FooBar`; `Baz` is matched on its own.
    pub type Computers = Product![
        ExtractFieldAndHandle<Symbol!("Baz"), HandleFieldValue<ShowBaz>>,
        DowncastAndHandle<FooBar, ShowFooBar>,
    ];

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: MatchWithHandlers<Computers>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: ((), FooBarBaz),
            }
        }
    }

    #[test]
    fn test_downcast_and_handle() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(app.compute(code, FooBarBaz::Foo(1)), "FooBar::Foo(1)");
        assert_eq!(app.compute(code, FooBarBaz::Baz(true)), "Baz(true)");
    }
}
