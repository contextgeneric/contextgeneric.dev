use core::fmt::Display;

use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

// error[E0207]: the type parameter `T` is not constrained by the impl trait, self type, or
//               predicates
//
// The registration impl copies the provider impl's generic parameters but drops its `where`
// clause, and nothing in `impl DefaultImpls1<ShowImplComponent, C> for String { type Delegate =
// ShowWithDisplay; }` mentions `T`.
#[cgp_impl(new ShowWithDisplay)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl<T: Display> ShowImpl<T> {
    fn show(&self, value: &T) -> String {
        format!("{value}")
    }
}

fn main() {}
