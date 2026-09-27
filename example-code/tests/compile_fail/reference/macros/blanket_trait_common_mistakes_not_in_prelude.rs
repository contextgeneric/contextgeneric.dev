//! `docs/reference/macros/blanket_trait.md`, *Common Mistakes*: the macro is not in the prelude.

use cgp::prelude::*;

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

fn main() {}
