//! Code from `docs/concepts/monadic-handlers.md` — *Monadic handlers*.

use cgp::extra::monad::monadic::err::ErrMonadic;
use cgp::extra::monad::providers::PipeMonadic;
use cgp::prelude::*;

/// A fallible step: succeeds with the next value, or reports an overflow.
#[cgp_computer]
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}

/// ## Plain composition is wrong for a fallible step
///
/// Three steps under the identity monad: the `Result` is threaded forward untouched, so the second
/// step receives a `Result` rather than the value inside it and the chain never short-circuits.
/// Written out, the type mismatch is what stops this compiling — which is the honest form of "plain
/// composition does not fit".
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/monadic_handlers_plain_composition_is_wrong.rs`.
pub mod plain_composition_is_wrong {}

/// ## Chaining through the err monad
pub mod chaining_through_the_err_monad {
    #[test]
    fn the_first_error_becomes_the_output() {
        use super::{ErrMonadic, Increment, PipeMonadic};
        use cgp::prelude::*;

        let context = ();
        let code = PhantomData::<()>;

        // Three successful steps.
        assert_eq!(
            PipeMonadic::<ErrMonadic, Product![Increment, Increment, Increment]>::compute(
                &context, code, 1u8,
            ),
            Ok(4),
        );

        // The third step overflows, and the remaining steps do not run.
        assert_eq!(
            PipeMonadic::<ErrMonadic, Product![Increment, Increment, Increment]>::compute(
                &context, code, 253u8,
            ),
            Err("overflow"),
        );
    }
}

/// ## The identity monad recovers plain composition
pub mod the_identity_monad_recovers_plain_composition {
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn double(value: u8) -> u8 {
        value * 2
    }

    #[test]
    fn nothing_short_circuits() {
        use cgp::extra::monad::monadic::ident::IdentMonadic;
        use cgp::extra::monad::providers::PipeMonadic;

        assert_eq!(
            PipeMonadic::<IdentMonadic, Product![Double, Double]>::compute(
                &(),
                PhantomData::<()>,
                3u8,
            ),
            12,
        );
    }
}

/// ## A pipeline is still an ordinary provider
///
/// The composed chain is wired like any other computation, so nothing downstream knows a monad was
/// involved.
pub mod a_pipeline_is_still_a_provider {
    use cgp::extra::handler::ComputerComponent;

    use super::*;

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: PipeMonadic<ErrMonadic, Product![Increment, Increment]>,
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
    fn the_context_just_computes() {
        use cgp::extra::handler::CanCompute;

        assert_eq!(App.compute(PhantomData::<()>, 1u8), Ok(3));
        assert_eq!(App.compute(PhantomData::<()>, 254u8), Err("overflow"));
    }
}
