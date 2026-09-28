use cgp::prelude::*;

pub struct Payload;
pub struct Other;

// error: ambiguous associated item
//
// Both the headline and the note land on the derive: the colliding variant is the generated
// companion's, whose identifiers carry the derive's span.
#[derive(ExtractField)]
pub enum Slot {
    Value(Payload),
    Other(Other),
}

fn main() {}
