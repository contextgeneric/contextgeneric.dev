use core::fmt::Display;

use cgp::prelude::*;

// error[E0425]: cannot find type `Name` in this scope
//
// The same failure as `impl_generics_rejected_parameter_in_signature.rs`, with the parameter named
// bare rather than as the qualifier of a path, which the compiler reports under a different code.
#[cgp_fn]
#[impl_generics(Name: Display + Clone)]
pub fn owned_name(&self, #[implicit] name: &Name) -> Name {
    name.clone()
}

fn main() {}
