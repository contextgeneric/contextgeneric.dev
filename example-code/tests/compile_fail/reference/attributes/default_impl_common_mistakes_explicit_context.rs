use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

// error[E0207]: the type parameter `Context` is not constrained by the impl trait, self type, or
//               predicates
#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl<Context> ShowImpl<String> for Context {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}

fn main() {}
