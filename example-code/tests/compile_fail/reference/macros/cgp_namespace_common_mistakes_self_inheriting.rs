use cgp::prelude::*;

// A namespace inheriting itself does not overflow; its forwarding impl has a value parameter
// nothing can determine.
cgp_namespace! { new A: A {} }

fn main() {}
