//! `docs/reference/macros/cgp_provider.md`, *Input the macros refuse*: a const argument in the
//! provider trait's argument list.

use cgp::prelude::*;

pub trait Sized2<Context, const N: usize> {
    fn size() -> usize;
}

pub struct Provider;

#[cgp_provider(SomeComponent)]
impl<Context> Sized2<Context, 3> for Provider {
    fn size() -> usize {
        3
    }
}

fn main() {}
