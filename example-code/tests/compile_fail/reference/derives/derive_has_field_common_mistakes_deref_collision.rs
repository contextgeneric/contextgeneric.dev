use core::ops::Deref;

use cgp::prelude::*;

#[derive(HasField)]
pub struct Inner {
    pub name: String,
}

// error[E0119]: conflicting implementations of trait `HasField<Symbol<4, …>>` for type `Outer`
#[derive(HasField)]
pub struct Outer {
    pub inner: Inner,
    pub name: String,
}

impl Deref for Outer {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.inner
    }
}

fn main() {}
