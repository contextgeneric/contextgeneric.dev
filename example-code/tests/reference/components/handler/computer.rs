//! Code from `docs/reference/components/handler/computer.md` — `Computer`.
//!
//! Pins the `Double` provider from the page's Examples, wired onto a context and invoked through
//! `CanCompute`.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::{CanCompute, Computer, ComputerComponent};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for Double {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    #[derive(HasField)]
    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: Double,
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
    fn app_doubles_its_input() {
        assert_eq!(App.compute(PhantomData::<()>, 21u64), 42);
    }
}
