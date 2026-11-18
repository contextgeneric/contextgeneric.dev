use cgp::prelude::*;

// A inherits B and B inherits A, so each forwarding impl's `where` clause requires the other and
// the trait solver overflows at both definitions.
cgp_namespace! { new A: B {} }

cgp_namespace! { new B: A {} }

fn main() {}
