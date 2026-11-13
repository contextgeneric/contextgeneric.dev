//! Runnable check for the code shown on `docs/quickstart.md`.
//!
//! The Quickstart is a standalone page rather than a section, so it gets its own test binary rather
//! than a module tree under a section entry point. The page's entire claim is that this program
//! compiles and runs, so that is what this checks. The page's `fn main` is a `#[test]` here; nothing
//! else differs from what the page shows, and `the_page_shows_this_program` holds the two equal.

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

/// The program between `use cgp::prelude::*;` and the end of this file, with the test's header put
/// back as the page's `fn main()`, must equal the page's `src/main.rs` block.
#[test]
fn the_page_shows_this_program() {
    use std::fs;
    use std::path::Path;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let page = fs::read_to_string(root.join("../docs/quickstart.md")).unwrap();
    let start = page.find("```rust title=\"src/main.rs\"\n").unwrap();
    let body = &page[start..];
    let body = &body[body.find('\n').unwrap() + 1..];
    let shown = &body[..body.find("```").unwrap()];

    let source = fs::read_to_string(root.join("tests/quickstart.rs")).unwrap();
    let from = source.find("use cgp::prelude::*;").unwrap();
    let to = source.find("\n/// The program between").unwrap();
    let compiled = source[from..to]
        .replace("#[test]\nfn quickstart_program_runs()", "fn main()");

    assert_eq!(shown.trim_end(), compiled.trim_end());
}
