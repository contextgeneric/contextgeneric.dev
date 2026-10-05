// From docs/cargo-cgp/reading-output.md, A fix in a help line.

use cgp::prelude::*;

#[cgp_fn]
fn person_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

#[cgp_fn]
fn greeting(&self) -> String {
    format!("Hello, {}!", self.person_name())
}

fn main() {}
