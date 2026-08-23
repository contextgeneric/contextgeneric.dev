//! Code from `docs/reference/providers/use_context.md` — *`UseContext`*.
//!
//! Pins the non-cyclic default-inner-provider use: a loud greeter wraps the context's own greeter
//! through `UseContext`, delegating to a *different* component so there is no cycle.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_component(LoudGreeter)]
    pub trait CanGreetLoudly {
        fn greet_loudly(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello".to_owned()
        }
    }

    pub struct GreetLoudly<Inner = UseContext>(pub PhantomData<Inner>);

    #[cgp_impl(GreetLoudly<Inner>)]
    #[use_provider(Inner: Greeter)]
    impl<Inner> LoudGreeter {
        fn greet_loudly(&self) -> String {
            format!("{}!", Inner::greet(self).to_uppercase())
        }
    }

    pub struct App;

    delegate_components! {
        App {
            GreeterComponent: GreetHello,
            LoudGreeterComponent: GreetLoudly,
        }
    }

    check_components! {
        App {
            GreeterComponent,
            LoudGreeterComponent,
        }
    }

    #[test]
    fn test_use_context_default_inner() {
        let app = App;
        assert_eq!(app.greet_loudly(), "HELLO!");
    }
}
