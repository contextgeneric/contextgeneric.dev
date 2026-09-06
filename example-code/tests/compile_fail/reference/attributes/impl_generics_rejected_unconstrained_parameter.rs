use core::fmt::Display;

use cgp::prelude::*;

// error[E0207]: the type parameter `Name` is not constrained by the impl trait, self type, or
//               predicates
#[cgp_fn]
#[impl_generics(Name: Display)]
pub fn greet(&self, #[implicit] label: &str) -> String {
    format!("Hello, {label}!")
}

fn main() {}
