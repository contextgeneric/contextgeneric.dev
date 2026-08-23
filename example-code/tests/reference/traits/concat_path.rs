//! Code from `docs/reference/traits/concat_path.md` — *`ConcatPath`*.
//!
//! Paths are unsized markers, so there is no value to build: naming the joined type and coercing it
//! to the expected one is the check. The absence of any `use` beyond the prelude is the page's claim
//! that this trait, unlike its neighbours, is re-exported there.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    pub type Outer = Path!(@a.b);
    pub type Inner = Path!(@c.d);

    pub type Joined = <Outer as ConcatPath<Inner>>::Output;

    #[allow(dead_code)]
    pub fn assert_joined(path: PhantomData<&Joined>) -> PhantomData<&Path!(@a.b.c.d)> {
        path
    }
}
