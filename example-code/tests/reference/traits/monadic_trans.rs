//! Code from `docs/reference/traits/monad/monadic_trans.md` — `MonadicTrans`.
//!
//! Pins the Examples program: what applying each shipped marker as a transformer produces, checked as
//! type equalities.

/// ## Examples
pub mod examples {
    use cgp::extra::monad::monadic::err::{ErrMonadic, ErrMonadicTrans};
    use cgp::extra::monad::monadic::ident::IdentMonadic;
    use cgp::extra::monad::monadic::ok::{OkMonadic, OkMonadicTrans};
    use cgp::extra::monad::traits::MonadicTrans;
    use cgp::prelude::*;

    // `IdentMonadic` leaves the base unchanged.
    pub fn ident(
        m: PhantomData<<IdentMonadic as MonadicTrans<ErrMonadic>>::M>,
    ) -> PhantomData<ErrMonadic> {
        m
    }

    // `OkMonadic` over `ErrMonadic`, the stack `PipeMonadic` builds for a fallible pipeline.
    pub fn ok_over_err(
        m: PhantomData<<OkMonadic as MonadicTrans<ErrMonadic>>::M>,
    ) -> PhantomData<OkMonadicTrans<ErrMonadic>> {
        m
    }

    // Transformers compose: the outer one wraps whatever the inner one produces.
    pub fn nested(
        m: PhantomData<<ErrMonadicTrans<OkMonadic> as MonadicTrans<IdentMonadic>>::M>,
    ) -> PhantomData<ErrMonadicTrans<OkMonadicTrans<IdentMonadic>>> {
        m
    }
}
