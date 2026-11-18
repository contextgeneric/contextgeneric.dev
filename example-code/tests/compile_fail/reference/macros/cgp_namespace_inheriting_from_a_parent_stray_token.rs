use cgp::prelude::*;

cgp_namespace! { new BaseNamespace {} }

// A header may be followed by a braced table or by nothing. A `;` is neither.
cgp_namespace! { new ExtendedNamespace: BaseNamespace; }

fn main() {}
