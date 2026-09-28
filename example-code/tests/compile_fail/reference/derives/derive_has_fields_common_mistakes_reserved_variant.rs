use cgp::prelude::*;

// error: ambiguous associated item
#[derive(HasFields)]
pub enum Message {
    Fields(u8),
    Other(u16),
}

fn main() {}
