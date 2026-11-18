//! `docs/reference/macros/cgp_impl.md`, *Common Mistakes*: `#[default_impl]` on the explicit
//! `impl<Context> Trait for Context` form copies an unconstrained `Context`.

use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
#[prefix(@test in DefaultNamespace)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl<Context> ShowImpl<String> for Context {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}

fn main() {}
