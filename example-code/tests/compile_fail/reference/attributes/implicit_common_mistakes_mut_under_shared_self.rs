use cgp::prelude::*;

// error: &mut self is required for mutable field reference `& mut u64`
#[cgp_fn]
fn bump(&self, #[implicit] counter: &mut u64) {
    *counter += 1;
}

fn main() {}
