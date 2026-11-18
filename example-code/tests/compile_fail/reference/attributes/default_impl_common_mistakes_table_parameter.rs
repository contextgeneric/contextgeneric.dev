use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

pub struct App;

// error[E0107]: trait takes 2 generic arguments but 3 generic arguments were supplied
#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent, App>)]
impl ShowImpl<String> {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}

fn main() {}
