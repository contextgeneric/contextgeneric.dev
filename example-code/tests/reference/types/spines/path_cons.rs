//! Code from `docs/reference/types/spines/path_cons.md` — *`PathCons`*.
//!
//! `PathCons` is built by `Path!` and emitted by `cgp_namespace!`. Paths are unsized markers, so there
//! is no value to build: naming the `PathCons` chain a `Path!` produces and coercing it is the check,
//! and the namespace registration pins that the `=>` redirect the page shows compiles.

/// ## Examples
///
/// A lowercase path segment is a `Symbol`, so `Path!(@a.b)` is a two-segment `PathCons` chain, which is
/// the expansion the page shows. The page's `cgp_namespace!` then reroutes one component to a path; the
/// two components it names are defined here, which the page elides, so the redirect compiles.
pub mod examples {
    use cgp::prelude::*;

    pub type TwoSegments = Path!(@a.b);

    #[allow(dead_code)]
    pub fn assert_path_is_pathcons(
        path: PhantomData<&TwoSegments>,
    ) -> PhantomData<&PathCons<Symbol!("a"), PathCons<Symbol!("b"), Nil>>> {
        path
    }

    #[cgp_component(FooProvider)]
    pub trait CanFoo {
        fn foo(&self);
    }

    #[cgp_component(MyFoo)]
    pub trait CanMyFoo {
        fn my_foo(&self);
    }

    cgp_namespace! {
        new MyNamespace {
            FooProviderComponent =>
                @MyFooComponent,
        }
    }
}
