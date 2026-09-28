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

pub struct App;

// error[E0119]: conflicting implementations of trait `DelegateComponent<…>` for type `App`
delegate_components! {
    App {
        for <T, Provider> in DefaultImpls1<ShowImplComponent> {
            @test.ShowImplComponent.T: Provider,
        }

        @test.ShowImplComponent.String: ShowString,
    }
}

fn main() {}
