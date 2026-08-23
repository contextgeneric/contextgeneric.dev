//! Code from `docs/reference/components/can_wrap_error.md` — `CanWrapError`.
//!
//! Pins the `LoadOrFail` provider from the page's Examples, which raises a `String` into the context's
//! abstract error and then wraps a further message onto it, declaring both dependencies through
//! `#[uses]`. The provider is generic over its context, so it compiles on its own; the
//! `CanRaiseError<String>` and `CanWrapError<String>` dependencies are discharged wherever a concrete
//! context wires `LoadOrFail`.

/// ## Examples
pub mod examples {
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
}
