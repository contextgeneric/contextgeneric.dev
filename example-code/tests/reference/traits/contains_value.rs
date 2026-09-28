//! Code from `docs/reference/traits/monad/contains_value.md` — `ContainsValue`.
//!
//! Pins the Examples program: the value each shipped marker reads beneath an output, including a
//! two-layer stack, checked as type equalities.

/// ## Examples
pub mod examples {
    use cgp::extra::monad::monadic::err::ErrMonadic;
    use cgp::extra::monad::monadic::ident::IdentMonadic;
    use cgp::extra::monad::monadic::ok::{OkMonadic, OkMonadicTrans};
    use cgp::extra::monad::traits::ContainsValue;

    pub fn ident(value: <IdentMonadic as ContainsValue<u8>>::Value) -> u8 {
        value
    }

    // `ErrMonadic` continues on `Ok`, so the value is the `Ok` payload.
    pub fn err(value: <ErrMonadic as ContainsValue<Result<u8, String>>>::Value) -> u8 {
        value
    }

    // `OkMonadic` continues on `Err`, so the value is the error.
    pub fn ok(value: <OkMonadic as ContainsValue<Result<u8, String>>>::Value) -> String {
        value
    }

    // The outer layer is `ErrMonadic`'s, the inner one the `Ok` layer's.
    pub fn stacked(
        value: <OkMonadicTrans<ErrMonadic> as ContainsValue<Result<Result<u8, String>, bool>>>::Value,
    ) -> String {
        value
    }
}
