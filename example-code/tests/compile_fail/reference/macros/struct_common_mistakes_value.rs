// From `docs/reference/macros/struct.md`, *Common mistakes*: a value written where `Struct!`
// expects a field type.

use cgp::prelude::*;

type Shape = Struct! { a: 1 };

fn main() {}
