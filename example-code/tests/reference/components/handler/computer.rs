//! Code from `docs/reference/components/handler/computer.md` — `Computer`.
//!
//! Pins the `Double` provider from the page's Examples, wired onto a context directly and through the
//! `PromoteComputer` bundle, and the wiring the page's Usage section shows for a context that joins
//! `DefaultNamespace`. The ambiguous associated-function call from Common Mistakes is a trybuild
//! fixture.

/// ## Usage
///
/// A context that joins `DefaultNamespace` binds the provider at the component's prefixed path.
pub mod usage {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanCompute;
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for Double {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace DefaultNamespace;

            @cgp.extra.handler.ComputerComponent: Double,
        }
    }

    mod check_app {
        use super::*;

        check_components! {
            App {
                ComputerComponent: ((), u64),
            }
        }
    }

    #[test]
    fn the_prefixed_path_binds_the_provider() {
        assert_eq!(App.compute(PhantomData::<()>, 21), 42);
    }
}

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanCompute, CanTryCompute};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for Double {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ComputerComponent: Double,
            TryComputerComponent: PromoteComputer<Double>,
        }
    }

    check_components! {
        App {
            [ComputerComponent, TryComputerComponent]: ((), u64),
        }
    }

    pub fn demo() {
        assert_eq!(App.compute(PhantomData::<()>, 21), 42);
        assert_eq!(App.try_compute(PhantomData::<()>, 21), Ok(42));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Common Mistakes
///
/// The fix the page gives for the ambiguous call: call through the consumer trait by name. The
/// rejected call is `tests/compile_fail/reference/components/computer_common_mistakes_ambiguous_concrete_call.rs`.
pub mod common_mistakes {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanCompute;
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for Double {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: Double,
        }
    }

    check_components! {
        App {
            ComputerComponent: ((), u64),
        }
    }

    #[test]
    fn the_qualified_call_resolves() {
        assert_eq!(App.compute(PhantomData::<()>, 21), 42);
        assert_eq!(
            <App as CanCompute<(), u64>>::compute(&App, PhantomData, 21),
            42
        );
    }
}
