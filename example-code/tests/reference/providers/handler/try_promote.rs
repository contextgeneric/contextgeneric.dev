//! Code from `docs/reference/providers/handler/try_promote.md` — `TryPromote`.
//!
//! Pins both directions the page shows: a `Computer` returning a `Result` serves a `TryComputer` slot,
//! and a `TryComputer` serves a `Computer` slot whose output is the `Result`.

/// ## Examples
pub mod examples {
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::{DebugError, RaiseFrom};
    use cgp::extra::handler::{CanCompute, CanTryCompute, TryPromote};
    use cgp::prelude::*;

    /// A computer whose output is a `Result` in the context's error type.
    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for CheckedDouble
    where
        Context: HasErrorType<Error = String>,
    {
        type Output = Result<u64, String>;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> Result<u64, String> {
            input.checked_mul(2).ok_or_else(|| "overflow".to_owned())
        }
    }

    /// A genuinely fallible computer.
    #[cgp_impl(new ParseU64)]
    #[uses(CanRaiseError<ParseIntError>)]
    #[use_type(HasErrorType.Error)]
    impl<Code> TryComputer<Code, String> {
        type Output = u64;

        fn try_compute(&self, _code: PhantomData<Code>, input: String) -> Result<u64, Error> {
            input.parse().map_err(Self::raise_error)
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            @ErrorRaiserComponent.String: RaiseFrom,
            @ErrorRaiserComponent.ParseIntError: DebugError,

            TryComputerComponent: TryPromote<CheckedDouble>,
            ComputerComponent: TryPromote<ParseU64>,
        }
    }

    check_components! {
        App {
            TryComputerComponent: ((), u64),
            ComputerComponent: ((), String),
        }
    }

    pub fn demo() {
        let code = PhantomData::<()>;

        // The `Result` output becomes the fallible interface.
        assert_eq!(App.try_compute(code, 21), Ok(42));
        assert_eq!(App.try_compute(code, u64::MAX), Err("overflow".to_owned()));

        // The fallible computer surfaces its `Result` as a plain value.
        assert_eq!(App.compute(code, "12".to_owned()), Ok(12));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
