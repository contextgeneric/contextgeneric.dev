use cgp::prelude::*;

// error: expect body to be either a struct or enum
#[derive(HasFields)]
pub union Bits {
    small: u8,
    large: u16,
}

fn main() {}
