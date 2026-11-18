use cgp::prelude::*;

// error: a `&mut` implicit argument must be the only implicit argument
#[cgp_fn]
fn bump(&mut self, #[implicit] counter: &mut u64, #[implicit] step: u64) {
    *counter += step;
}

fn main() {}
