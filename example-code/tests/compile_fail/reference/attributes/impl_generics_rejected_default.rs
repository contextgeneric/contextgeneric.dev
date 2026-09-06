use core::fmt::Display;

use cgp::prelude::*;

// error: defaults for generic parameters are not allowed here
#[cgp_fn]
#[impl_generics(Total: Display = u32)]
pub fn describe_total(&self, #[implicit] total: &Total) -> String {
    format!("{total}")
}

fn main() {}
