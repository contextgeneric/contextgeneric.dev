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

pub struct NoClone;

// error[E0277]: the trait bound `NoClone: Clone` is not satisfied
pub fn scale_it<Ctx>(ctx: &Ctx) -> NoClone
where
    Ctx: Scale<NoClone>,
{
    ctx.scale()
}

fn main() {}
