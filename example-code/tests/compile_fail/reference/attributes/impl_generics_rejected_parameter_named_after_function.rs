use core::fmt::Display;

use cgp::prelude::*;

// error[E0404]: expected trait, found type parameter `Count`
//
// The generated trait is named `Count` after the function, and so is the parameter, so inside the
// generated impl the trait's name resolves to the parameter.
#[cgp_fn]
#[impl_generics(Count: Display)]
pub fn count(&self, #[implicit] count: &Count) -> String {
    format!("{count}")
}

fn main() {}
