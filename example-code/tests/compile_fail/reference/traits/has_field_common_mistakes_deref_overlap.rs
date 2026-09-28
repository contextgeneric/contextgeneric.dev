use core::ops::Deref;

use cgp::prelude::*;

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

// `Wrapper` derefs to `Person`, which has `name`, so the forwarding impl already covers that tag.
#[derive(HasField)]
pub struct Wrapper {
    pub name: String,
    pub person: Person,
}

impl Deref for Wrapper {
    type Target = Person;

    fn deref(&self) -> &Person {
        &self.person
    }
}

fn main() {}
