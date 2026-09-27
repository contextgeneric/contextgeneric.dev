use cgp::prelude::*;

// error[E0369] and others: `Symbol!` builds a type, so it fails in expression position
pub fn tag() {
    let _tag = Symbol!("name");
}

fn main() {}
