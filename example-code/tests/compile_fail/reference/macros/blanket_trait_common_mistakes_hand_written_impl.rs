//! `docs/reference/macros/blanket_trait.md`, *Common Mistakes*: the blanket impl conflicts with a hand-written impl for a type that satisfies its bounds.

use cgp::core::macros::blanket_trait;

pub trait Foo {
    fn foo(&self);
}

#[blanket_trait]
pub trait FooExt: Foo {
    fn foo_twice(&self) {
        self.foo();
        self.foo();
    }
}

pub struct Ctx;

impl Foo for Ctx {
    fn foo(&self) {}
}

impl FooExt for Ctx {
    fn foo_twice(&self) {}
}

fn main() {}
