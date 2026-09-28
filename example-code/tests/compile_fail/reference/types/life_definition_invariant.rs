// `Life<'a>` holds a `PhantomData<*mut &'a ()>`, and `*mut T` is invariant in `T`, so a `Life<'l>`
// does not convert to a `Life<'s>` even when `'l` outlives `'s`. This pins the `Life` page's
// *Definition*; the same function over a covariant `PhantomData<&'l ()>` compiles.

use cgp::prelude::*;

pub fn shorten<'s, 'l: 's>(marker: Life<'l>) -> Life<'s> {
    marker
}

fn main() {}
