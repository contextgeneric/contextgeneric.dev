// A tuple field is keyed by `Index<N>`, never by a string of its digit, so `Symbol!("0")` names no
// field of a tuple struct. This is the `Index` page's *Common Mistakes* entry on the two tags.

use cgp::prelude::*;

#[derive(HasField)]
pub struct Pair(pub u32, pub String);

pub fn first(pair: &Pair) -> u32 {
    *pair.get_field(PhantomData::<Symbol!("0")>)
}

fn main() {}
