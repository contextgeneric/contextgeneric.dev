use cgp::prelude::*;

// The macro rejects a namespace inheriting itself.
cgp_namespace! { new A: A {} }

fn main() {}
