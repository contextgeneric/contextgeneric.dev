//! Code from `docs/reference/components/can_raise_error.md` — `CanRaiseError`.
//!
//! Pins the `LoadOrFail` provider from the page's Examples, which raises a `String` into the context's
//! abstract error through `#[uses(CanRaiseError<String>)]`. The provider is generic over its context,
//! so it compiles on its own; the `CanRaiseError<String>` dependency is discharged wherever a concrete
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
    #[uses(CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Loader {
        fn load(&self, path: &str) -> Result<String, Error> {
            if path.is_empty() {
                return Err(Self::raise_error("empty path".to_owned()));
            }
            Ok(format!("contents of {path}"))
        }
    }
}
