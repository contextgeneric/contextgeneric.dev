//! Code from `docs/reference/traits/formatting/concat_path.md` — `ConcatPath`.
//!
//! Paths are unsized markers, so there is no value to build: the check is a type equality, and the
//! prelude-only import pins that the trait is re-exported there. `Path!` without its `@` is a
//! trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    pub type Outer = Path!(@a.b);
    pub type Inner = Path!(@c.d);

    pub type Joined = <Outer as ConcatPath<Inner>>::Output;

    // Compiles only if the joined path is `@a.b.c.d`.
    pub fn assert_joined(path: PhantomData<Joined>) -> PhantomData<Path!(@a.b.c.d)> {
        path
    }

    // A generic signature naming the composed route.
    pub fn descend<Outer: ?Sized, Inner: ?Sized>(
    ) -> PhantomData<<Outer as ConcatPath<Inner>>::Output>
    where
        Outer: ConcatPath<Inner>,
    {
        PhantomData
    }
}
