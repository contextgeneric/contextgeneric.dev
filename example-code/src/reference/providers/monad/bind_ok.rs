//! Code from `docs/reference/providers/monad/bind_ok.md` — `BindOk`.

/// ## Usage and Examples
///
/// `BindOk` is the mirror of `BindErr`: it runs its continuation on the `Err` value and
/// short-circuits on `Ok`, which is the fallback behavior `PipeMonadic` composes under `OkMonadic`.
/// The page shows it built by hand inside a `PipeHandlers` list.
pub mod one_hand_built_bind_step {
    use cgp::prelude::*;

    /// Accept a small value, or hand a too-large one to the fallback as an `Err`.
    #[cgp_computer]
    pub fn classify(value: u8) -> Result<u8, u8> {
        if value < 10 {
            Ok(value)
        } else {
            Err(value)
        }
    }

    /// The fallback: run only on the `Err` branch, halving the rejected value.
    #[cgp_computer]
    pub fn halve(value: u8) -> Result<u8, u8> {
        Ok(value / 2)
    }

    #[test]
    fn the_bind_runs_the_continuation_on_the_err_value() {
        use cgp::extra::handler::PipeHandlers;
        use cgp::extra::monad::monadic::ident::IdentMonadic;
        use cgp::extra::monad::monadic::ok::BindOk;

        // 5 is accepted, so BindOk short-circuits and the fallback never runs.
        assert_eq!(
            PipeHandlers::<Product![Classify, BindOk<IdentMonadic, Halve>]>::compute(
                &(),
                PhantomData::<()>,
                5u8,
            ),
            Ok(5),
        );

        // 20 is rejected, so BindOk runs the fallback on the Err value: 20 -> 10.
        assert_eq!(
            PipeHandlers::<Product![Classify, BindOk<IdentMonadic, Halve>]>::compute(
                &(),
                PhantomData::<()>,
                20u8,
            ),
            Ok(10),
        );
    }
}
