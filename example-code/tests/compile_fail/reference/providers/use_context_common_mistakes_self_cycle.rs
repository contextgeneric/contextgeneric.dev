use cgp::prelude::*;

#[cgp_component(Encoder)]
pub trait CanEncode<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}

pub struct App;

delegate_components! {
    App {
        open EncoderComponent;

        @EncoderComponent.u32: UseContext,
    }
}

check_components! {
    App {
        EncoderComponent: u32,
    }
}

fn main() {}
