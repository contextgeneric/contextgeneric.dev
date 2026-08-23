use cgp::extra::handler::{Computer, PipeHandlers};
use cgp::prelude::*;

#[cgp_computer]
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}

fn main() {
    // `Increment` produces `Result<u8, _>` and the next `Increment` wants a `u8`, so plain
    // composition does not type-check.
    let _ = PipeHandlers::<Product![Increment, Increment]>::compute(&(), PhantomData::<()>, 1u8);
}
