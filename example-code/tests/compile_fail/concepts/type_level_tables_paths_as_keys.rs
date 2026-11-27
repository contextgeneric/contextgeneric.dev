use cgp::prelude::*;

#[cgp_component(Converter)]
pub trait CanConvert<Source, Target> {
    fn convert(&self, source: Source) -> Target;
}

#[cgp_impl(new ConvertFromU8)]
impl<Target: From<u8>> Converter<u8, Target> {
    fn convert(&self, source: u8) -> Target {
        Target::from(source)
    }
}

#[cgp_impl(new ConvertU8ToU64)]
impl Converter<u8, u64> {
    fn convert(&self, source: u8) -> u64 {
        source as u64
    }
}

pub struct App;

delegate_components! {
    App {
        open ConverterComponent;

        // The shorter key matches every path beneath it, `@ConverterComponent.u8.u64` included.
        @ConverterComponent.u8: ConvertFromU8,
        @ConverterComponent.u8.u64: ConvertU8ToU64,
    }
}

fn main() {}
