//! Code from `docs/reference/providers/monad/bind_err.md` — `BindErr`.
//!
//! Pins the Examples program: one err-monad bind step built by hand inside a `PipeHandlers` list,
//! which is what `PipeMonadic` composes internally under `ErrMonadic`.

/// ## Examples
pub mod examples {
    use cgp::extra::handler::PipeHandlers;
    use cgp::extra::monad::monadic::err::BindErr;
    use cgp::extra::monad::monadic::ident::IdentMonadic;
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn increment(value: u8) -> Result<u8, &'static str> {
        value.checked_add(1).ok_or("overflow")
    }

    pub type Pipeline = PipeHandlers<Product![Increment, BindErr<IdentMonadic, Increment>]>;

    pub fn demo() {
        let code = PhantomData::<()>;

        // 1 -> Ok(2); BindErr runs the second Increment on 2 -> Ok(3).
        assert_eq!(Pipeline::compute(&(), code, 1), Ok(3));
        // The first step overflows, so the bind skips the second.
        assert_eq!(Pipeline::compute(&(), code, 255), Err("overflow"));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
