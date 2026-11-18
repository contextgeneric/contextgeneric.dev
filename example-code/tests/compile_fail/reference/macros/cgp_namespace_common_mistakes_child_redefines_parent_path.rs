use cgp::prelude::*;

pub struct Seg;

cgp_namespace! {
    new BaseNamespace {
        @app => @base,
    }
}

// error[E0119]: the child's more specific path overlaps the parent's, whose key ends in a wildcard
cgp_namespace! {
    new ExtendedNamespace: BaseNamespace {
        @app.Seg => @child,
    }
}

fn main() {}
