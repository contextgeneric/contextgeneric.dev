use cgp::prelude::*;

// error[E0214]: parenthesized type parameters may only be used with a `Fn` trait
pub type Row = product![u32, bool];

fn main() {}
