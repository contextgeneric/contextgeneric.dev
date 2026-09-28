//! Code from `docs/reference/components/can_wrap_error.md` — `CanWrapError`.
//!
//! The page's Definition carries `#[track_caller]`, which the `cgp` revision this crate pins predates;
//! the caller-location claims were checked against the local `cgp` checkout instead. The circular
//! `DisplayError` wiring and the ambiguous concrete-context call are trybuild fixtures.

/// ## Usage and Examples
///
/// The `open` table from Usage and the context from Examples, one program: `AppendDetail` folds a
/// `String` detail into the error, and `DisplayError` forwards a `u64` detail to it.
pub mod examples {
    use cgp::core::error::{
        ErrorRaiserComponent, ErrorTypeProviderComponent, ErrorWrapper, ErrorWrapperComponent,
    };
    use cgp::extra::error::{DisplayError, RaiseFrom};
    use cgp::prelude::*;

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self, path: &str) -> Result<String, Error>;
    }

    #[cgp_impl(new LoadOrFail)]
    #[uses(CanRaiseError<String>, CanWrapError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Loader {
        fn load(&self, path: &str) -> Result<String, Error> {
            if path.is_empty() {
                let err = Self::raise_error("empty path".to_owned());
                return Err(Self::wrap_error(err, format!("while loading {path}")));
            }
            Ok(format!("contents of {path}"))
        }
    }

    #[cgp_impl(new AppendDetail)]
    #[use_type(HasErrorType.{Error = String})]
    impl ErrorWrapper<String> {
        fn wrap_error(error: Error, detail: String) -> Error {
            format!("{detail}: {error}")
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,
            ErrorWrapperComponent: AppendDetail,
            LoaderComponent: LoadOrFail,
        }
    }

    check_components! {
        App {
            LoaderComponent,
        }
    }

    /// The Usage table: `String` goes to `AppendDetail`, and `u64` through `DisplayError` to it.
    pub struct UsageApp;

    delegate_components! {
        UsageApp {
            open ErrorWrapperComponent;

            ErrorTypeProviderComponent: UseType<String>,
            @ErrorWrapperComponent.String: AppendDetail,
            @ErrorWrapperComponent.u64: DisplayError,
        }
    }

    check_components! {
        UsageApp {
            ErrorWrapperComponent: [String, u64],
        }
    }

    #[test]
    fn detail_is_folded_into_the_error() {
        assert_eq!(App.load(""), Err("while loading : empty path".to_owned()));
        assert_eq!(
            <UsageApp as CanWrapError<u64>>::wrap_error("failed".to_owned(), 7),
            "7: failed"
        );
    }
}
