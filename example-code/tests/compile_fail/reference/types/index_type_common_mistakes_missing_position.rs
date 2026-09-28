// A position a tuple struct lacks has no `HasField` impl, so reading it is a compile error rather
// than a runtime panic. This is the `Index` page's *Common Mistakes* entry on an absent position.

use cgp::prelude::*;

#[derive(HasField)]
pub struct Point(pub f64, pub f64, pub f64);

pub fn sixth(point: &Point) -> f64 {
    *point.get_field(PhantomData::<Index<5>>)
}

fn main() {}
