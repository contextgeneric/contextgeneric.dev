//! Code from `docs/reference/providers/redirect_lookup.md` — *`RedirectLookup`*.
//!
//! `RedirectLookup` is generated machinery, so the code the page shows is the namespace registration
//! that produces a redirect, and the components whose generated impls Under the hood lists. The
//! bare-key entry from Common Mistakes is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    #[prefix(@app in DefaultNamespace)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "hello".into()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace DefaultNamespace;

            @app.GreeterComponent: GreetHello,
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    pub fn demo() {
        assert_eq!(App.greet(), "hello");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Under the hood
///
/// The components whose `RedirectLookup` impls the page lists, for `cargo cgp expand`.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }
}
