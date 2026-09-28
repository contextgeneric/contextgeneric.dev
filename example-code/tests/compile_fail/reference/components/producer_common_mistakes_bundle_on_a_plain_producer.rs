use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;

#[cgp_new_provider]
impl<Context, Code> Producer<Context, Code> for MagicNumber {
    type Output = u64;

    fn produce(_context: &Context, _code: PhantomData<Code>) -> u64 {
        42
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        [ComputerComponent, TryComputerComponent]: PromoteProducer<MagicNumber>,
    }
}

check_components! {
    App {
        [ComputerComponent, TryComputerComponent]: ((), ()),
    }
}

fn main() {}
