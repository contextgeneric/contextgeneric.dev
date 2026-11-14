use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::ComputerComponent;
use cgp::extra::handler::PromoteTryComputer;
use cgp::extra::handler::{Computer, HandlerComponent};
use cgp::prelude::*;

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for CheckedDouble
where
    Context: HasErrorType<Error = String>,
{
    type Output = Result<u64, String>;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> Result<u64, String> {
        input.checked_mul(2).ok_or_else(|| "overflow".to_owned())
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        HandlerComponent: PromoteTryComputer<CheckedDouble>,
    }
}

check_components! {
    App {
        HandlerComponent: ((), u64),
    }
}

fn main() {}
