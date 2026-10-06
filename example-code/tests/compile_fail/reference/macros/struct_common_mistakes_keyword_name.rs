// From `docs/reference/macros/struct.md`, *Common Mistakes*: a keyword field name written without
// its `r#`, which the macro reads as a positional field.

use cgp::prelude::*;

type Shape = Struct! { type: u8 };

fn main() {}
