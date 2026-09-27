use cgp::prelude::*;

// error: expected one of: `for`, parentheses, `fn`, ...
pub fn row() {
    let _row = Product![1, 2];
}

fn main() {}
