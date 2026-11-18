//! Code from `docs/reference/providers/error/discard_detail.md` — `DiscardDetail`.
//!
//! Pins the Examples program: an error is raised, wrapped with a detail string, and the wrapper
//! `DiscardDetail` drops the detail so the base error propagates unchanged.

/// ## Examples
pub mod examples {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent, ErrorWrapperComponent};
    use cgp::extra::error::{DiscardDetail, RaiseFrom};
    use cgp::prelude::*;

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self) -> Result<(), Error>;
    }

    #[cgp_impl(new LoadConfig)]
    #[uses(CanRaiseError<String>, CanWrapError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Loader {
        fn load(&self) -> Result<(), Error> {
            let error = Self::raise_error("disk offline".to_owned());
            Err(Self::wrap_error(error, "while loading config".to_owned()))
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,
            ErrorWrapperComponent: DiscardDetail,
            LoaderComponent: LoadConfig,
        }
    }

    check_components! {
        App {
            LoaderComponent,
        }
    }

    pub fn demo() {
        assert_eq!(App.load(), Err("disk offline".to_owned()));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
