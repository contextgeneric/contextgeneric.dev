use cgp::prelude::*;

// error: this relaxed bound is not permitted here
#[cgp_fn]
#[uses(?Sized)]
pub fn thing(&self) -> u8 {
    1
}

fn main() {}
