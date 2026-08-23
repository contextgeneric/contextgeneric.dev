//! Code from `docs/reference/providers/error/discard_detail.md` — `DiscardDetail`.

/// ## Usage and Examples
///
/// `DiscardDetail` satisfies the wrapping capability by returning the error and dropping the detail.
/// Here a base error is raised and then wrapped with a detail string; because the wrapper is
/// `DiscardDetail`, the detail is discarded and the base error propagates unchanged.
pub mod wrapping_by_discarding {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent, ErrorWrapperComponent};
    use cgp::extra::error::{DiscardDetail, RaiseFrom};
    use cgp::prelude::*;

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self) -> Result<(), Error>;
    }

    #[cgp_impl(new LoadWithContext)]
    #[uses(CanRaiseError<String>, CanWrapError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Loader {
        fn load(&self) -> Result<(), Error> {
            let base = Self::raise_error("disk offline".to_owned());
            // The detail is attached here, but the wrapper is free to drop it.
            Err(Self::wrap_error(base, "while loading config".to_owned()))
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            ErrorWrapperComponent: DiscardDetail,
            LoaderComponent: LoadWithContext,

            @ErrorRaiserComponent.String: RaiseFrom,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { LoaderComponent } }
    }

    #[test]
    fn the_detail_is_dropped_and_the_error_is_unchanged() {
        assert_eq!(App.load().unwrap_err(), "disk offline");
    }
}
