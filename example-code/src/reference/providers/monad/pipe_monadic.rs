//! Code from `docs/reference/providers/monad/pipe_monadic.md` — `PipeMonadic`.

/// ## Usage and Examples
///
/// `PipeMonadic` composes a handler list under a monad into one short-circuiting handler. Under
/// `ErrMonadic` each step runs on the previous step's `Ok` value and the first `Err` becomes the
/// output. The composed pipeline is wired like any other computation.
pub mod chaining_under_a_monad {
    use cgp::extra::handler::ComputerComponent;
    use cgp::extra::monad::monadic::err::ErrMonadic;
    use cgp::extra::monad::providers::PipeMonadic;
    use cgp::prelude::*;

    /// A fallible step: succeeds with the next value, or reports an overflow.
    #[cgp_computer]
    pub fn increment(value: u8) -> Result<u8, &'static str> {
        value.checked_add(1).ok_or("overflow")
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: PipeMonadic<ErrMonadic, Product![Increment, Increment, Increment]>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: ((), u8),
            }
        }
    }

    #[test]
    fn the_first_error_becomes_the_output() {
        use cgp::extra::handler::CanCompute;

        // Three successful steps.
        assert_eq!(App.compute(PhantomData::<()>, 1u8), Ok(4));

        // The third step overflows, and the remaining steps do not run.
        assert_eq!(App.compute(PhantomData::<()>, 253u8), Err("overflow"));
    }

    #[test]
    fn invoked_directly_without_wiring() {
        assert_eq!(
            PipeMonadic::<ErrMonadic, Product![Increment, Increment, Increment]>::compute(
                &App,
                PhantomData::<()>,
                1u8,
            ),
            Ok(4),
        );
    }
}
