//! Code from `docs/reference/traits/monad/monadic_bind.md` — `MonadicBind`.
//!
//! Pins the Examples program: the bind provider each shipped marker wraps a continuation in,
//! checked as type equalities.

/// ## Examples
pub mod examples {
    use cgp::extra::monad::monadic::err::{BindErr, ErrMonadic};
    use cgp::extra::monad::monadic::ident::IdentMonadic;
    use cgp::extra::monad::monadic::ok::{BindOk, OkMonadic};
    use cgp::extra::monad::traits::MonadicBind;
    use cgp::prelude::*;

    // A continuation provider; any type stands in for one here.
    pub struct Next;

    // Each function compiles only if the projection is the type on its right.
    pub fn ident(
        p: PhantomData<<IdentMonadic as MonadicBind<Next>>::Provider>,
    ) -> PhantomData<Next> {
        p
    }

    pub fn err(
        p: PhantomData<<ErrMonadic as MonadicBind<Next>>::Provider>,
    ) -> PhantomData<BindErr<IdentMonadic, Next>> {
        p
    }

    pub fn ok(
        p: PhantomData<<OkMonadic as MonadicBind<Next>>::Provider>,
    ) -> PhantomData<BindOk<IdentMonadic, Next>> {
        p
    }
}
