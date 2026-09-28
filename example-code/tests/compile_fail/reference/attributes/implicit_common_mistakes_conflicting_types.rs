use cgp::prelude::*;

#[cgp_component(Counter)]
pub trait CanCount {
    fn small(&self) -> u32;
    fn big(&self) -> u64;
}

// error[E0284]: type annotations needed: cannot satisfy `<__Context__ as HasField<…>>::Value == u32`
#[cgp_impl(new ReadCount)]
impl Counter {
    fn small(&self, #[implicit] count: u32) -> u32 {
        count
    }

    fn big(&self, #[implicit] count: u64) -> u64 {
        count
    }
}

fn main() {}
