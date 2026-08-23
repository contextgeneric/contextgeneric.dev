//! Code from `docs/reference/components/handler/try_computer.md` — `TryComputer`.
//!
//! Pins the `ParseU64` provider from the page's Examples, wired onto a context that also supplies an
//! error type and an error raiser for `ParseIntError`, and invoked through `CanTryCompute`.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::{DebugError, RaiseFrom};
    use cgp::extra::handler::{TryComputer, TryComputerComponent};
    use cgp::prelude::*;

    #[cgp_impl(new ParseU64)]
    #[uses(CanRaiseError<core::num::ParseIntError>)]
    #[use_type(HasErrorType.Error)]
    impl<Code> TryComputer<Code, String> {
        type Output = u64;

        fn try_compute(
            &self,
            _code: PhantomData<Code>,
            input: String,
        ) -> Result<Self::Output, Error> {
            input.parse().map_err(|e| Self::raise_error(e))
        }
    }

    pub struct App;

    // The page shows only `TryComputerComponent: ParseU64` and says the context also wires an error
    // type and an error raiser. That backend is filled in here so the wiring resolves: `DebugError`
    // formats the `ParseIntError` into a `String`, which `RaiseFrom` then raises into the `String`
    // error type.
    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            TryComputerComponent: ParseU64,
            @ErrorRaiserComponent.core::num::ParseIntError: DebugError,
            @ErrorRaiserComponent.String: RaiseFrom,
        }
    }

    mod check_app {
        use super::*;

        check_components! {
            App {
                TryComputerComponent: ((), String),
            }
        }
    }

    #[test]
    fn app_parses_a_number() {
        use cgp::extra::handler::CanTryCompute;

        assert_eq!(
            App.try_compute(PhantomData::<()>, "42".to_owned()).unwrap(),
            42,
        );
    }
}
