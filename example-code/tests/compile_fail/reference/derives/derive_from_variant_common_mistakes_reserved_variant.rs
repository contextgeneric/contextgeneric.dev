use cgp::prelude::*;

pub struct Payload;

// error: ambiguous associated item
#[derive(FromVariant)]
pub enum Slot {
    Value(Payload),
}

fn main() {}
