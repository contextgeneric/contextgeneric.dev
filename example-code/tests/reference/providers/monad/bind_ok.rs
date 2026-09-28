//! Code from `docs/reference/providers/monad/bind_ok.md` — `BindOk`.
//!
//! Pins the Examples program: one ok-monad bind step built by hand inside a `PipeHandlers` list, which
//! runs a fallback on the `Err` value and stops on `Ok`.

/// ## Examples
pub mod examples {
    use cgp::extra::handler::PipeHandlers;
    use cgp::extra::monad::monadic::ident::IdentMonadic;
    use cgp::extra::monad::monadic::ok::BindOk;
    use cgp::prelude::*;

    /// Accept a small value, or hand a too-large one to the fallback as an `Err`.
    #[cgp_computer]
    pub fn classify(value: u8) -> Result<u8, u8> {
        if value < 10 { Ok(value) } else { Err(value) }
    }

    /// The fallback, run only on the `Err` branch: halve the rejected value.
    #[cgp_computer]
    pub fn halve(value: u8) -> Result<u8, u8> {
        Ok(value / 2)
    }

    pub type Pipeline = PipeHandlers<Product![Classify, BindOk<IdentMonadic, Halve>]>;

    pub fn demo() {
        let code = PhantomData::<()>;

        // 5 is accepted, so BindOk stops and the fallback never runs.
        assert_eq!(Pipeline::compute(&(), code, 5), Ok(5));
        // 20 is rejected, so BindOk runs the fallback on the Err value: 20 -> 10.
        assert_eq!(Pipeline::compute(&(), code, 20), Ok(10));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
