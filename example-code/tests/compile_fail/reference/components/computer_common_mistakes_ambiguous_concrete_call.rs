use core::marker::PhantomData;

use cgp::extra::handler::CanCompute;
use cgp::prelude::*;

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: Double,
    }
}

fn main() {
    let _ = App::compute(&App, PhantomData::<()>, 21);
}
