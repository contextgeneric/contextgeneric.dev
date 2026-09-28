use core::marker::PhantomData;

use cgp::extra::handler::PromoteRef;
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
        ComputerRefComponent: PromoteRef<Double>,
    }
}

check_components! {
    App {
        ComputerRefComponent: ((), u64),
    }
}

fn main() {}
