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

// error[E0277]: the trait bound `Scalar: Clone` is not satisfied
pub fn scale_it<Ctx, Scalar>(ctx: &Ctx) -> Scalar
where
    Ctx: Scale<Scalar>,
{
    ctx.scale()
}

fn main() {}
