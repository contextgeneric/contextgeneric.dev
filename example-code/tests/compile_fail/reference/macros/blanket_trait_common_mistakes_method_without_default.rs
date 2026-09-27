//! `docs/reference/macros/blanket_trait.md`, *Common Mistakes*: a method needs a default body.

use cgp::core::macros::blanket_trait;

pub trait Foo {
    fn foo(&self);
}

#[blanket_trait]
pub trait FooExt: Foo {
    fn foo_twice(&self);
}

fn main() {}
