use cgp::prelude::*;

pub struct Payload;
pub struct Other;

// error: ambiguous associated item
#[derive(CgpVariant)]
pub enum Slot {
    Remainder(Payload),
    Other(Other),
}

fn main() {}
