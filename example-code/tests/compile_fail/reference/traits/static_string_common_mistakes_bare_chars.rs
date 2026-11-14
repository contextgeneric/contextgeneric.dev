use cgp::core::base::types::Chars;
use cgp::core::field::traits::StaticString;
use cgp::core::field::types::Nil;

// A bare character list, without the `Symbol` wrapper that carries its byte length.
pub const NAME: &str = <Chars<'a', Nil> as StaticString>::VALUE;

fn main() {}
