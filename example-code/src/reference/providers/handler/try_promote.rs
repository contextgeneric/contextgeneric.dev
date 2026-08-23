//! Code from `docs/reference/providers/handler/try_promote.md` — `TryPromote`.

/// ## Usage and Examples
///
/// `TryPromote` turns a `Computer` that *returns* a `Result` into a genuine `TryComputer`, unwrapping
/// the `Result` into the fallible interface. The context's error type is what the `Result`'s error
/// arm becomes.
pub mod unwrapping_a_result_output {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{TryComputerComponent, TryPromote};
    use cgp::prelude::*;

    /// A computer whose output is itself a `Result`.
    #[cgp_computer]
    pub fn checked_add(a: u64, b: u64) -> Result<u64, String> {
        a.checked_add(b).ok_or_else(|| "overflow".to_owned())
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,
            TryComputerComponent: TryPromote<CheckedAdd>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                TryComputerComponent: ((), (u64, u64)),
            }
        }
    }

    #[test]
    fn the_result_becomes_the_fallible_interface() {
        use cgp::extra::handler::CanTryCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(app.try_compute(code, (1u64, 2u64)), Ok(3));
        assert_eq!(app.try_compute(code, (u64::MAX, 1u64)), Err("overflow".to_owned()));
    }
}
