use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}

// error[E0119]: conflicting implementations of trait `DefaultImpls1<ShowImplComponent, _>` for
//               type `String`
#[cgp_impl(new ShowStringLoudly)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> {
    fn show(&self, value: &String) -> String {
        value.to_uppercase()
    }
}

fn main() {}
