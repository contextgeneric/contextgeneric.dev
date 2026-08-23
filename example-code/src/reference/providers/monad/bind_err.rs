//! Code from `docs/reference/providers/monad/bind_err.md` — `BindErr`.

/// ## Usage and Examples
///
/// `BindErr` is one bind step of the err monad: it runs its continuation on the `Ok` value and
/// short-circuits on `Err`. This is what `PipeMonadic` composes internally under `ErrMonadic`; the
/// page shows it built by hand inside a `PipeHandlers` list, which is the one time it is named
/// directly.
pub mod one_hand_built_bind_step {
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn increment(value: u8) -> Result<u8, &'static str> {
        value.checked_add(1).ok_or("overflow")
    }

    #[test]
    fn the_bind_runs_the_continuation_on_the_ok_value() {
        use cgp::extra::handler::PipeHandlers;
        use cgp::extra::monad::monadic::err::BindErr;
        use cgp::extra::monad::monadic::ident::IdentMonadic;

        // 1 -> Ok(2); BindErr runs the second Increment on 2 -> Ok(3).
        assert_eq!(
            PipeHandlers::<Product![Increment, BindErr<IdentMonadic, Increment>]>::compute(
                &(),
                PhantomData::<()>,
                1u8,
            ),
            Ok(3),
        );

        // The first step overflows, so the bind short-circuits and the continuation never runs.
        assert_eq!(
            PipeHandlers::<Product![Increment, BindErr<IdentMonadic, Increment>]>::compute(
                &(),
                PhantomData::<()>,
                255u8,
            ),
            Err("overflow"),
        );
    }
}
