//! Code from `docs/reference/components/handler/try_computer_ref.md` — `TryComputerRef`.
//!
//! Pins the `ParsePort` provider from the page's Examples, which parses a borrowed string, wired onto a
//! context that supplies its error type and raiser.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{CanTryComputeRef, TryComputerRef};
    use cgp::prelude::*;

    #[cgp_impl(new ParsePort)]
    #[uses(CanRaiseError<ParseIntError>)]
    #[use_type(HasErrorType.Error)]
    impl<Code> TryComputerRef<Code, String> {
        type Output = u16;

        fn try_compute_ref(
            &self,
            _code: PhantomData<Code>,
            input: &String,
        ) -> Result<Self::Output, Error> {
            input.trim().parse().map_err(Self::raise_error)
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
            TryComputerRefComponent: ParsePort,
        }
    }

    check_components! {
        App {
            TryComputerRefComponent: ((), String),
        }
    }

    pub fn demo() {
        let setting = " 8080 ".to_owned();

        assert_eq!(App.try_compute_ref(PhantomData::<()>, &setting), Ok(8080));
        assert!(App.try_compute_ref(PhantomData::<()>, &"http".to_owned()).is_err());
        // The caller still owns `setting`.
        assert_eq!(setting.len(), 6);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
