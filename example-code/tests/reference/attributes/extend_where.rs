//! Code from `docs/reference/attributes/extend_where.md` — *`#[extend_where]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/attributes/`.

/// ## Overview, Usage, and Examples
///
/// The example trait, a caller who states the promoted predicate, and the wider predicate forms
/// the grammar accepts.
pub mod examples {
    use core::ops::Mul;

    use cgp::prelude::*;

    #[cgp_fn]
    #[extend_where(Scalar: Clone)]
    pub fn scale<Scalar>(&self, #[implicit] factor: Scalar) -> Scalar
    where
        Scalar: Mul<Output = Scalar>,
    {
        factor.clone() * factor
    }

    pub fn scale_it<Ctx, Scalar>(ctx: &Ctx) -> Scalar
    where
        Ctx: Scale<Scalar>,
        Scalar: Clone,
    {
        ctx.scale()
    }

    #[cgp_fn]
    #[extend_where(for<'a> &'a Items: IntoIterator<Item = &'a u32>, Items: Default)]
    pub fn total<Items>(&self, #[implicit] items: &Items) -> u32 {
        items.into_iter().sum()
    }

    #[derive(HasField)]
    pub struct Scaler {
        pub factor: u32,
        pub items: Vec<u32>,
    }

    #[test]
    fn the_caller_states_the_bound() {
        let scaler = Scaler {
            factor: 3,
            items: vec![1, 2],
        };
        assert_eq!(scale_it::<_, u32>(&scaler), 9);
        assert_eq!(Total::<Vec<u32>>::total(&scaler), 3);
    }
}

/// ## Examples: the bound left on the implementation
///
/// With `Clone` in the function's own `where` clause, a caller naming the trait for a type that
/// cannot be cloned compiles, since nothing checks the bound until a context is supplied.
pub mod left_on_the_implementation {
    use core::ops::Mul;

    use cgp::prelude::*;

    #[cgp_fn]
    pub fn scale<Scalar>(&self, #[implicit] factor: Scalar) -> Scalar
    where
        Scalar: Mul<Output = Scalar> + Clone,
    {
        factor.clone() * factor
    }

    pub struct NoClone;

    pub fn scale_it<Ctx>(ctx: &Ctx) -> NoClone
    where
        Ctx: Scale<NoClone>,
    {
        ctx.scale()
    }
}

/// ## Under the hood
///
/// The listing's input, and one function combining every contributor to the implementation's
/// `where` clause, to pin their order.
pub mod under_the_hood {
    use core::ops::Mul;

    use cgp::prelude::*;

    #[cgp_fn]
    #[extend_where(Scalar: Clone)]
    pub fn scale<Scalar>(&self, #[implicit] factor: Scalar) -> Scalar
    where
        Scalar: Mul<Output = Scalar>,
    {
        factor.clone() * factor
    }

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_fn]
    #[use_type(HasErrorType.Error)]
    #[extend_where(Scalar: Clone)]
    #[uses(HasName)]
    pub fn ordered<Scalar>(&self, #[implicit] factor: Scalar) -> Result<Scalar, Error>
    where
        Scalar: Mul<Output = Scalar>,
    {
        Ok(factor.clone() * factor)
    }
}
