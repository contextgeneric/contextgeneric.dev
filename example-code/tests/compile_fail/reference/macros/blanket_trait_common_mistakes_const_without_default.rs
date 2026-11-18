//! `docs/reference/macros/blanket_trait.md`, *Common Mistakes*: a constant needs a default expression.

use cgp::core::macros::blanket_trait;

pub trait Foo {}

#[blanket_trait]
pub trait FooExt: Foo {
    const LIMIT: u32;
}

fn main() {}
