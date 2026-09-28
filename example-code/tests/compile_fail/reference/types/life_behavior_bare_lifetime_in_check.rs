// A check lists a lifetime component's parameters as `(Life<'a>, T)`. A bare `'a` in the tuple is
// read as a trait-object type without a trait. This pins the `Life` page's *Behavior*.

use cgp::prelude::*;

#[cgp_component(ReferenceGetter)]
pub trait HasReference<'a, T: 'a + ?Sized> {
    fn get_reference(&self) -> &'a T;
}

pub struct Config {
    pub name: String,
}

pub struct App<'a> {
    pub config: &'a Config,
}

#[cgp_impl(new GetConfig)]
impl<'a> ReferenceGetter<'a, Config> for App<'a> {
    fn get_reference(&self) -> &'a Config {
        self.config
    }
}

delegate_components! {
    <'a> App<'a> {
        ReferenceGetterComponent: GetConfig,
    }
}

check_components! {
    <'a> App<'a> {
        ReferenceGetterComponent: ('a, Config),
    }
}

fn main() {}
