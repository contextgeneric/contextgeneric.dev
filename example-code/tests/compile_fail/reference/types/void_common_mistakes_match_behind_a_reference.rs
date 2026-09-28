// A match on a sum by value may leave out the `Void` arm, but a match behind a reference may not:
// Rust does not treat an uninhabited type reached through a reference as unreachable. This is the
// `Void` page's *Common Mistakes* entry on the closing arm.

use cgp::prelude::*;

pub type Token = Sum![u32, bool];

pub fn describe(token: &Token) -> String {
    match token {
        Either::Left(number) => format!("number {number}"),
        Either::Right(Either::Left(flag)) => format!("flag {flag}"),
    }
}

fn main() {}
