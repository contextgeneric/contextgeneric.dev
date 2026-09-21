//! Runnable check for the code shown on `docs/quickstart.md`.
//!
//! The Quickstart is a standalone page rather than a section, so it gets its own test binary rather
//! than a module tree under a section entry point. The page's entire claim is that this program
//! compiles and runs, so that is what this checks. The page's `fn main` is a `#[test]` here; nothing
//! else differs from what the page shows.

use cgp::prelude::*;

#[cgp_fn]
pub fn greet(&self, #[implicit] name: &str) {
    println!("Hello, {name}!");
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

#[test]
fn quickstart_program_runs() {
    let person = Person {
        name: "World".to_owned(),
    };

    person.greet();
}
