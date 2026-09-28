//! Code from `docs/reference/components/handler/try_computer.md` — `TryComputer`.
//!
//! Pins the `ParseU64` provider from the page's Examples, wired onto a context that supplies its error
//! type and raiser, and the `#[cgp_computer]` function whose `Result` output the page's Usage section
//! describes.

/// ## Usage
///
/// A `Result`-returning function becomes a `Computer` whose output is the `Result`, and its
/// `PromoteTryComputer` bundle reads that `Result` as the failure path.
pub mod usage {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanCompute, CanTryCompute};
    use cgp::prelude::*;

    #[cgp_computer]
    fn checked_double(input: u64) -> Result<u64, String> {
        input.checked_mul(2).ok_or_else(|| "overflow".to_owned())
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            [ComputerComponent, TryComputerComponent]: CheckedDouble,
        }
    }

    check_components! {
        App {
            [ComputerComponent, TryComputerComponent]: ((), u64),
        }
    }

    #[test]
    fn the_result_is_the_computer_output_and_the_try_computer_failure() {
        assert_eq!(App.compute(PhantomData::<()>, 21), Ok(42));
        assert_eq!(App.try_compute(PhantomData::<()>, 21), Ok(42));
        assert_eq!(
            App.try_compute(PhantomData::<()>, u64::MAX),
            Err("overflow".to_owned())
        );
    }
}

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::CanTryCompute;
    use cgp::prelude::*;

    #[cgp_impl(new ParseU64)]
    #[uses(CanRaiseError<ParseIntError>)]
    #[use_type(HasErrorType.Error)]
    impl<Code> TryComputer<Code, String> {
        type Output = u64;

        fn try_compute(
            &self,
            _code: PhantomData<Code>,
            input: String,
        ) -> Result<Self::Output, Error> {
            input.parse().map_err(Self::raise_error)
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct AppError(pub String);

    impl From<ParseIntError> for AppError {
        fn from(e: ParseIntError) -> Self {
            AppError(e.to_string())
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<AppError>,
            ErrorRaiserComponent: RaiseFrom,
            TryComputerComponent: ParseU64,
        }
    }

    check_components! {
        App {
            TryComputerComponent: ((), String),
        }
    }

    pub fn demo() {
        assert_eq!(App.try_compute(PhantomData::<()>, "12".to_owned()), Ok(12));
        assert!(App.try_compute(PhantomData::<()>, "twelve".to_owned()).is_err());
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
