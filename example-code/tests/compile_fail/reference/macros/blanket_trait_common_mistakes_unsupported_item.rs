//! `docs/reference/macros/blanket_trait.md`, *Common Mistakes*: only methods, associated types, and constants are accepted.

use cgp::core::macros::blanket_trait;

pub trait Foo {}

macro_rules! no_items {
    () => {};
}

#[blanket_trait]
pub trait FooExt: Foo {
    no_items!();
}

fn main() {}
