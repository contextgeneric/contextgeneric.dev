//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: a context whose field has another name does not satisfy the getter.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Person {
    pub full_name: String,
}

pub fn greet(person: &Person) -> &str {
    person.name()
}

fn main() {}
