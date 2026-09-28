use cgp::prelude::*;

// error: relaxed bounds are not permitted in supertrait bounds
#[cgp_fn]
#[extend(?Sized)]
pub fn thing(&self) -> u8 {
    1
}

fn main() {}
